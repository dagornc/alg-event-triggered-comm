use std::process::Command;

#[test]
fn cli_emet_un_entete_et_une_ligne_csv() {
    let output = Command::new(env!("CARGO_BIN_EXE_event_triggered_comm_rs"))
        .args(["1001", "30", "20", "6", "2", "3", "5", "nominal"])
        .output()
        .expect("exécution CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].split(',').count(), lines[1].split(',').count());
}

#[test]
fn cli_refuse_un_scenario_inconnu() {
    let status = Command::new(env!("CARGO_BIN_EXE_event_triggered_comm_rs"))
        .args(["1001", "30", "20", "6", "2", "3", "5", "inconnu"])
        .status()
        .expect("exécution CLI");
    assert!(!status.success());
}
