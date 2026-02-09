use std::fs;
use vig::analyzer::{AnalyzeResult, PortDirection, VhdlType, analyze_vhdl};

fn analyze_file(path: &str) -> AnalyzeResult {
    let source = fs::read_to_string(path).unwrap_or_else(|_| panic!("failed to read {}", path));
    analyze_vhdl(&source).expect("analysis failed")
}

#[test]
fn test_counter_entity_found() {
    let result = analyze_file("testdata/counter.vhd");
    assert_eq!(result.entities.len(), 1);
    assert_eq!(result.entities[0].name, "counter");
}

#[test]
fn test_counter_entity_port_count() {
    let result = analyze_file("testdata/counter.vhd");
    assert_eq!(result.entities[0].ports.len(), 3);
}

#[test]
fn test_counter_entity_port_details() {
    let result = analyze_file("testdata/counter.vhd");
    let ports = &result.entities[0].ports;

    // clk : in std_logic
    assert_eq!(ports[0].name, "clk");
    assert_eq!(ports[0].direction, PortDirection::In);
    assert_eq!(ports[0].vhdl_type, VhdlType::StdLogic);

    // reset : in std_logic
    assert_eq!(ports[1].name, "reset");
    assert_eq!(ports[1].direction, PortDirection::In);
    assert_eq!(ports[1].vhdl_type, VhdlType::StdLogic);

    // count : out std_logic_vector(7 downto 0)
    assert_eq!(ports[2].name, "count");
    assert_eq!(ports[2].direction, PortDirection::Out);
    assert_eq!(
        ports[2].vhdl_type,
        VhdlType::StdLogicVector { high: 7, low: 0 }
    );
}

#[test]
fn test_counter_architecture_found() {
    let result = analyze_file("testdata/counter.vhd");
    assert_eq!(result.architectures.len(), 1);
    assert_eq!(result.architectures[0].name, "behavioral");
    assert_eq!(result.architectures[0].entity_name, "counter");
}

#[test]
fn test_counter_architecture_signal() {
    let result = analyze_file("testdata/counter.vhd");
    let signals = &result.architectures[0].signals;

    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].name, "counter_value");
    assert_eq!(signals[0].vhdl_type, VhdlType::Integer);
    assert_eq!(signals[0].default_value, Some("0".to_string()));
}

#[test]
fn test_entity_architecture_mapping() {
    let result = analyze_file("testdata/counter.vhd");
    assert_eq!(result.entities[0].name, result.architectures[0].entity_name);
}

#[test]
fn test_empty_entity() {
    let source = "entity empty_ent is\nend entity empty_ent;";
    let result = analyze_vhdl(source).unwrap();
    assert_eq!(result.entities.len(), 1);
    assert_eq!(result.entities[0].name, "empty_ent");
    assert!(result.entities[0].ports.is_empty());
}

#[test]
fn test_signal_without_default() {
    let source = r#"
        architecture rtl of foo is
            signal bar : std_logic;
        begin
        end architecture rtl;
    "#;
    let result = analyze_vhdl(source).unwrap();
    let sig = &result.architectures[0].signals[0];
    assert_eq!(sig.name, "bar");
    assert_eq!(sig.vhdl_type, VhdlType::StdLogic);
    assert_eq!(sig.default_value, None);
}

#[test]
fn test_multiple_entities() {
    let source = r#"
        entity a is
            port ( x : in std_logic );
        end entity a;
        entity b is
            port ( y : out integer );
        end entity b;
    "#;
    let result = analyze_vhdl(source).unwrap();
    assert_eq!(result.entities.len(), 2);
    assert_eq!(result.entities[0].name, "a");
    assert_eq!(result.entities[1].name, "b");
}

#[test]
fn test_case_insensitivity() {
    let source = "ENTITY MyEnt IS\nEND ENTITY MyEnt;";
    let result = analyze_vhdl(source).unwrap();
    assert_eq!(result.entities[0].name, "myent");
}
