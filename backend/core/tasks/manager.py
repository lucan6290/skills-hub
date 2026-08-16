"""Lightweight in-process task manager for long-running backend operations."""
from __future__ import annotations

import threading
import time
import traceback
import uuid
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from typing import Any, Callable, Optional
from contextlib import contextmanager


TaskFn = Callable[["TaskContext"], Any]


@dataclass
class TaskRecord:
    id: str
    kind: str
    status: str = "pending"
    progress: int = 0
    message: str = ""
    result: Any = None
    error: Optional[str] = None
    logs: list[str] = field(default_factory=list)
    cancel_requested: bool = False
    created_at: int = field(default_factory=lambda: int(time.time() * 1000))
    started_at: Optional[int] = None
    finished_at: Optional[int] = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "kind": self.kind,
            "status": self.status,
            "progress": self.progress,
            "message": self.message,
            "result": self.result,
            "error": self.error,
            "logs": self.logs[-200:],
            "cancel_requested": self.cancel_requested,
            "created_at": self.created_at,
            "started_at": self.started_at,
            "finished_at": self.finished_at,
        }


class TaskContext:
    def __init__(self, manager: "TaskManager", task_id: str):
        self._manager = manager
        self.task_id = task_id

    def log(self, message: str) -> None:
        self._manager.log(self.task_id, message)

    def set_progress(self, progress: int, message: str = "") -> None:
        self._manager.set_progress(self.task_id, progress, message)

    def is_cancelled(self) -> bool:
        task = self._manager.get(self.task_id)
        return bool(task and task.cancel_requested)

    def raise_if_cancelled(self) -> None:
        if self.is_cancelled():
            raise TaskCancelled("任务已取消 / task cancelled")

    def exclusive(self, message: str = "waiting for exclusive operation lock"):
        return self._manager.exclusive(self.task_id, message)


class TaskCancelled(RuntimeError):
    pass


class TaskManager:
    def __init__(self, max_workers: int = 3):
        self._tasks: dict[str, TaskRecord] = {}
        self._lock = threading.RLock()
        self._exclusive_lock = threading.RLock()
        self._executor = ThreadPoolExecutor(max_workers=max_workers, thread_name_prefix="skills-hub-task")

    def submit(self, kind: str, fn: TaskFn) -> TaskRecord:
        task_id = str(uuid.uuid4())
        record = TaskRecord(id=task_id, kind=kind)
        with self._lock:
            self._tasks[task_id] = record
        self._executor.submit(self._run, task_id, fn)
        return record

    def _run(self, task_id: str, fn: TaskFn) -> None:
        ctx = TaskContext(self, task_id)
        with self._lock:
            task = self._tasks[task_id]
            task.status = "running"
            task.started_at = int(time.time() * 1000)
            task.message = "running"
        try:
            result = fn(ctx)
            with self._lock:
                task = self._tasks[task_id]
                if task.cancel_requested:
                    task.status = "canceled"
                    task.message = "cancelled"
                else:
                    task.status = "succeeded"
                    task.progress = 100
                    task.message = "completed"
                    task.result = result
                task.finished_at = int(time.time() * 1000)
        except TaskCancelled as e:
            with self._lock:
                task = self._tasks[task_id]
                task.status = "canceled"
                task.message = str(e)
                task.finished_at = int(time.time() * 1000)
        except Exception as e:
            with self._lock:
                task = self._tasks[task_id]
                task.status = "failed"
                task.error = str(e)
                task.message = "failed"
                task.logs.append(traceback.format_exc())
                task.finished_at = int(time.time() * 1000)

    def get(self, task_id: str) -> Optional[TaskRecord]:
        with self._lock:
            return self._tasks.get(task_id)

    def list(self) -> list[TaskRecord]:
        with self._lock:
            return sorted(self._tasks.values(), key=lambda t: t.created_at, reverse=True)

    def cancel(self, task_id: str) -> bool:
        with self._lock:
            task = self._tasks.get(task_id)
            if not task:
                return False
            if task.status in {"succeeded", "failed", "canceled"}:
                return True
            task.cancel_requested = True
            task.message = "cancellation requested"
            return True

    def cancel_all_running(self) -> int:
        count = 0
        with self._lock:
            for task in self._tasks.values():
                if task.status in {"pending", "running"}:
                    task.cancel_requested = True
                    task.message = "cancellation requested"
                    count += 1
        return count

    def log(self, task_id: str, message: str) -> None:
        with self._lock:
            task = self._tasks.get(task_id)
            if task:
                task.logs.append(message)
                task.message = message

    def set_progress(self, task_id: str, progress: int, message: str = "") -> None:
        with self._lock:
            task = self._tasks.get(task_id)
            if task:
                task.progress = max(0, min(100, int(progress)))
                if message:
                    task.message = message

    @contextmanager
    def exclusive(self, task_id: str, message: str = "waiting for exclusive operation lock"):
        self.log(task_id, message)
        with self._exclusive_lock:
            task = self.get(task_id)
            if task and task.cancel_requested:
                raise TaskCancelled("任务已取消 / task cancelled")
            self.log(task_id, "exclusive operation lock acquired")
            yield


_manager: Optional[TaskManager] = None


def get_task_manager() -> TaskManager:
    global _manager
    if _manager is None:
        _manager = TaskManager()
    return _manager
