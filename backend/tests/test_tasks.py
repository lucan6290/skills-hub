import time

from core.tasks.manager import TaskManager


def test_task_manager_cancel_marks_running_task_canceled():
    manager = TaskManager(max_workers=1)

    def run(ctx):
        while True:
            ctx.raise_if_cancelled()
            time.sleep(0.01)

    task = manager.submit("loop", run)
    time.sleep(0.05)
    assert manager.cancel(task.id) is True

    deadline = time.time() + 2
    while time.time() < deadline:
        current = manager.get(task.id)
        if current and current.status == "canceled":
            break
        time.sleep(0.02)

    assert manager.get(task.id).status == "canceled"
