use std::{
    fs::{read_to_string, File},
    io::Write,
    process::{Command, Output},
};

fn create_file(path: &str) {
    File::create(path).unwrap();
}

fn create_test_dir(path: &str) {
    std::fs::create_dir_all(path).expect("failed to create dir");
}

fn delete_test_dir(path: &str) {
    let _ = std::fs::remove_dir_all(path);
}

fn cleanup_test_dir(path: &str) {
    delete_test_dir(path);
    create_test_dir(path);
}

#[allow(unused)]
pub fn build_result_from_output(output: Output) -> String {
    let mut result = Vec::new();
    result.write_all(&output.stdout).unwrap();
    result.write_all(&output.stderr).unwrap();

    String::from_utf8(strip_ansi_escapes::strip(String::from_utf8(result).unwrap()).unwrap())
        .unwrap()
}

#[test]
fn warning_message_on_init_output() {
    cleanup_test_dir("tests/.warning_message_on_init_output");
    create_file("tests/.warning_message_on_init_output/cluster.genin.yml");

    let output = Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("init")
    .arg("-q")
    .current_dir("tests/.warning_message_on_init_output")
    .output()
    .expect("Failed to execute command");

    let mut result = Vec::new();
    result.write_all(&output.stdout).unwrap();
    result.write_all(&output.stderr).unwrap();

    println!("{}", String::from_utf8(result.clone()).unwrap());

    assert_eq!(
        result,
        b"WARN: the target file cluster.genin.yml already exists so the new file will \
            be saved with name cluster.genin.copy.yml\n"
    );
}

#[test]
fn warning_message_on_build_output() {
    cleanup_test_dir("tests/.warning_message_on_build_output");
    create_file("tests/.warning_message_on_build_output/inventory.yml");

    Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("init")
    .arg("-q")
    .current_dir("tests/.warning_message_on_build_output")
    .output()
    .expect("Failed to execute command");

    let output = Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("build")
    .arg("--source")
    .arg("cluster.genin.yml")
    .arg("-q")
    .current_dir("tests/.warning_message_on_build_output")
    .output()
    .expect("Failed to execute command");

    let mut result = Vec::new();
    result.write_all(&output.stdout).unwrap();
    result.write_all(&output.stderr).unwrap();

    println!("{}", String::from_utf8(result.clone()).unwrap());

    assert_eq!(
        result,
        b"WARN: the target file inventory.yml already exists so the new file will be \
            saved with name inventory.copy.yml\n"
    );
}

#[test]
fn init_with_comments() {
    cleanup_test_dir("tests/.init_with_comments");

    Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("init")
    .arg("-q")
    .current_dir("tests/.init_with_comments")
    .output()
    .expect("Failed to execute command");

    let generated = std::fs::read_to_string("tests/.init_with_comments/cluster.genin.yml").unwrap();

    insta::assert_display_snapshot!(generated)
}

#[test]
fn build_from_state() {
    cleanup_test_dir("tests/.build_from_state");

    Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("build")
    .arg("-s")
    .arg("tests/resources/cluster.genin.yml")
    .arg("-o")
    .arg("tests/.build_from_state/inventory.yml")
    .arg("--export-state")
    .arg("tests/.build_from_state/state.json")
    .output()
    .expect("Failed to execute command");

    Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("build")
    .arg("-s")
    .arg("tests/.build_from_state/state.json")
    .arg("-o")
    .arg("tests/.build_from_state/inventory.yml")
    .arg("-f")
    .output()
    .expect("Failed to execute command");

    let inventory = read_to_string("tests/.build_from_state/inventory.yml").unwrap();

    insta::assert_display_snapshot!(inventory);

    Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("build")
    .arg("-s")
    .arg("tests/resources/cluster-new.genin.yml")
    .arg("-o")
    .arg("tests/.build_from_state/inventory_new.yml")
    .arg("--export-state")
    .arg("tests/.build_from_state/state.json")
    .output()
    .expect("Failed to execute command");

    Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("build")
    .arg("-s")
    .arg("tests/.build_from_state/state.json")
    .arg("-o")
    .arg("tests/.build_from_state/inventory_new.yml")
    .arg("-f")
    .output()
    .expect("Failed to execute command");

    let inventory_new = read_to_string("tests/.build_from_state/inventory_new.yml").unwrap();

    insta::assert_display_snapshot!(inventory_new);
}

#[test]
fn sequential_upgrade_from_state() {
    cleanup_test_dir("tests/.sequential_upgrade_from_state");

    let output = Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("upgrade")
    .arg("--old")
    .arg("tests/resources/cluster.genin.yml")
    .arg("--new")
    .arg("tests/resources/cluster-new.genin.yml")
    .arg("--output")
    .arg("tests/.sequential_upgrade_from_state/v1_inventory.yml")
    .arg("--export-state")
    .arg("tests/.sequential_upgrade_from_state/v1_state.json")
    .arg("--state-dir")
    .arg("tests/.sequential_upgrade_from_state/.geninstate")
    .output()
    .expect("Failed to execute command");

    let cluster_to_cluster_new = build_result_from_output(output);

    let cluster_to_cluster_new = format!(
        "{cluster_to_cluster_new}\n{}",
        read_to_string("tests/.sequential_upgrade_from_state/v1_inventory.yml").unwrap()
    );

    insta::assert_display_snapshot!("cluster_to_cluster_new", cluster_to_cluster_new);

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // upgrade from previous saved state
    ///////////////////////////////////////////////////////////////////////////////////////////////

    let output = Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("upgrade")
    .arg("--old")
    .arg("tests/.sequential_upgrade_from_state/v1_state.json")
    .arg("--new")
    .arg("tests/resources/cluster-new-v2.genin.yml")
    .arg("--output")
    .arg("tests/.sequential_upgrade_from_state/v2_inventory.yml")
    .arg("--state-dir")
    .arg("tests/.sequential_upgrade_from_state/.geninstate")
    .output()
    .expect("Failed to execute command");

    let cluster_new_to_cluster_v2 = build_result_from_output(output);

    let cluster_new_to_cluster_v2 = format!(
        "{cluster_new_to_cluster_v2}\n{}",
        read_to_string("tests/.sequential_upgrade_from_state/v2_inventory.yml").unwrap()
    );

    insta::assert_display_snapshot!("cluster_new_to_cluster_v2", cluster_new_to_cluster_v2);

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // upgrade from latest state
    ///////////////////////////////////////////////////////////////////////////////////////////////

    let output = Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("upgrade")
    .arg("--from-latest-state")
    .arg("--new")
    .arg("tests/resources/cluster-new-v3.genin.yml")
    .arg("--output")
    .arg("tests/.sequential_upgrade_from_state/v3_inventory.yml")
    .arg("--state-dir")
    .arg("tests/.sequential_upgrade_from_state/.geninstate")
    .output()
    .expect("Failed to execute command");

    let cluster_v2_to_cluster_v3 = build_result_from_output(output);
    let cluster_v2_to_cluster_v3 = format!(
        "{cluster_v2_to_cluster_v3}\n{}",
        read_to_string("tests/.sequential_upgrade_from_state/v3_inventory.yml").unwrap()
    );

    insta::assert_display_snapshot!("cluster_v2_to_cluster_v3", cluster_v2_to_cluster_v3);
}

#[test]
fn sequential_upgrade_with_decreasing() {
    cleanup_test_dir("tests/.sequential_upgrade_with_decreasing");

    let output = Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("upgrade")
    .arg("--old")
    .arg("tests/resources/cluster-new-v3.genin.yml")
    .arg("--new")
    .arg("tests/resources/cluster-new-v4.genin.yml")
    .arg("--output")
    .arg("tests/.sequential_upgrade_with_decreasing/v1_inventory.yml")
    .arg("--state-dir")
    .arg("tests/.sequential_upgrade_with_decreasing/.geninstate")
    .output()
    .expect("Failed to execute command");

    let cluster_v3_to_cluster_v4 = build_result_from_output(output);

    let cluster_v3_to_cluster_v4 = format!(
        "{cluster_v3_to_cluster_v4}\n{}",
        read_to_string("tests/.sequential_upgrade_with_decreasing/v1_inventory.yml").unwrap()
    );

    insta::assert_display_snapshot!("cluster_v3_to_cluster_v4", cluster_v3_to_cluster_v4);

    ///////////////////////////////////////////////////////////////////////////////////////////////
    // upgrade from latest state
    ///////////////////////////////////////////////////////////////////////////////////////////////

    let output = Command::new(format!(
        "{}/target/debug/genin",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .arg("upgrade")
    .arg("--from-latest-state")
    .arg("--new")
    .arg("tests/resources/cluster-new-v5.genin.yml")
    .arg("--output")
    .arg("tests/.sequential_upgrade_with_decreasing/v2_inventory.yml")
    .arg("--state-dir")
    .arg("tests/.sequential_upgrade_with_decreasing/.geninstate")
    .output()
    .expect("Failed to execute command");

    let cluster_v4_to_cluster_v5 = build_result_from_output(output);

    let cluster_v4_to_cluster_v5 = format!(
        "{cluster_v4_to_cluster_v5}\n{}",
        read_to_string("tests/.sequential_upgrade_with_decreasing/v2_inventory.yml").unwrap()
    );

    insta::assert_display_snapshot!("cluster_v4_to_cluster_v5", cluster_v4_to_cluster_v5);
}
