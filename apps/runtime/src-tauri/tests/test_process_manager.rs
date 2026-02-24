use runtime_lib::agent::tools::process_manager::ProcessManager;
use std::thread;
use std::time::Duration;

#[test]
fn test_spawn_echo_and_get_output() {
    let pm = ProcessManager::new();

    // 在后台执行一个快速命令
    let command = if cfg!(target_os = "windows") {
        "echo hello_pm"
    } else {
        "echo hello_pm"
    };
    let id = pm.spawn(command, None).unwrap();

    // 短 ID 应为 8 字符
    assert_eq!(id.len(), 8);

    // 阻塞等待输出
    let output = pm.get_output(&id, true).unwrap();
    assert!(output.exited);
    assert_eq!(output.exit_code, Some(0));
    assert!(output.stdout.contains("hello_pm"));
}

#[test]
fn test_spawn_and_kill() {
    let pm = ProcessManager::new();

    // 启动一个长时间运行的命令
    let command = if cfg!(target_os = "windows") {
        "ping -n 100 127.0.0.1"
    } else {
        "sleep 100"
    };
    let id = pm.spawn(command, None).unwrap();

    // 短暂等待确保进程已启动
    thread::sleep(Duration::from_millis(500));

    // 此时进程应该还在运行
    let output = pm.get_output(&id, false).unwrap();
    assert!(!output.exited);

    // 终止进程
    pm.kill(&id).unwrap();

    // 等待进程退出
    thread::sleep(Duration::from_millis(500));

    // 确认已退出
    let output = pm.get_output(&id, false).unwrap();
    assert!(output.exited);
}

#[test]
fn test_list_processes() {
    let pm = ProcessManager::new();

    let command = if cfg!(target_os = "windows") {
        "echo list_test"
    } else {
        "echo list_test"
    };
    let id = pm.spawn(command, None).unwrap();

    // 等待完成
    thread::sleep(Duration::from_millis(1000));

    let list = pm.list();
    assert!(!list.is_empty());

    // 找到我们的进程
    let found = list.iter().find(|(pid, _, _)| pid == &id);
    assert!(found.is_some());
    let (_, cmd, _) = found.unwrap();
    assert!(cmd.contains("echo"));
}

#[test]
fn test_get_output_nonexistent_process() {
    let pm = ProcessManager::new();

    let result = pm.get_output("nonexist", false);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("不存在"));
}

#[test]
fn test_kill_nonexistent_process() {
    let pm = ProcessManager::new();

    let result = pm.kill("nonexist");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("不存在"));
}

#[test]
fn test_cleanup() {
    let pm = ProcessManager::new();

    // 启动一个快速命令
    let command = if cfg!(target_os = "windows") {
        "echo cleanup_test"
    } else {
        "echo cleanup_test"
    };
    let id = pm.spawn(command, None).unwrap();

    // 等待完成
    let _ = pm.get_output(&id, true).unwrap();

    // cleanup 不应 panic
    pm.cleanup();

    // 进程仍应可查询（不超过上限不会被清理）
    let output = pm.get_output(&id, false).unwrap();
    assert!(output.exited);
}
