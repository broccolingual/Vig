use std::fs;
use vig::analyzer::analyze_vhdl;
use vig::generator::{TbConfig, generate_testbench};

fn gen_tb_from_file(path: &str) -> Vec<String> {
    let source = fs::read_to_string(path).unwrap_or_else(|_| panic!("failed to read {}", path));
    let result = analyze_vhdl(&source).expect("analysis failed");
    let config = TbConfig::default();
    result
        .entities
        .iter()
        .map(|e| generate_testbench(e, &config))
        .collect()
}

// === counter.vhd テスト ===

#[test]
fn test_counter_tb_entity_name() {
    let tbs = gen_tb_from_file("testdata/counter.vhd");
    assert_eq!(tbs.len(), 1);
    assert!(tbs[0].contains("entity counter_tb is"));
    assert!(tbs[0].contains("end entity counter_tb;"));
}

#[test]
fn test_counter_tb_component_declaration() {
    let tbs = gen_tb_from_file("testdata/counter.vhd");
    let tb = &tbs[0];
    assert!(tb.contains("component counter is"));
    assert!(tb.contains("clk : in std_logic"));
    assert!(tb.contains("reset : in std_logic"));
    assert!(tb.contains("count : out std_logic_vector(7 downto 0)"));
    assert!(tb.contains("end component counter;"));
}

#[test]
fn test_counter_tb_signal_declarations() {
    let tbs = gen_tb_from_file("testdata/counter.vhd");
    let tb = &tbs[0];
    assert!(tb.contains("signal clk : std_logic := '0';"));
    assert!(tb.contains("signal reset : std_logic := '0';"));
    assert!(tb.contains("signal count : std_logic_vector(7 downto 0) := (others => '0');"));
}

#[test]
fn test_counter_tb_dut_instance() {
    let tbs = gen_tb_from_file("testdata/counter.vhd");
    let tb = &tbs[0];
    assert!(tb.contains("uut: counter"));
    assert!(tb.contains("port map"));
    assert!(tb.contains("clk => clk"));
    assert!(tb.contains("reset => reset"));
    assert!(tb.contains("count => count"));
}

#[test]
fn test_counter_tb_clock_process() {
    let tbs = gen_tb_from_file("testdata/counter.vhd");
    let tb = &tbs[0];
    assert!(tb.contains("clk_process: process"));
    assert!(tb.contains("clk <= '0'"));
    assert!(tb.contains("clk <= '1'"));
    assert!(tb.contains("wait for 5 ns;"));
}

#[test]
fn test_counter_tb_reset_sequence() {
    let tbs = gen_tb_from_file("testdata/counter.vhd");
    let tb = &tbs[0];
    assert!(tb.contains("reset <= '1';"));
    assert!(tb.contains("reset <= '0';"));
}

#[test]
fn test_counter_tb_stimulus_process() {
    let tbs = gen_tb_from_file("testdata/counter.vhd");
    let tb = &tbs[0];
    assert!(tb.contains("stim_process: process"));
    assert!(tb.contains("assert false report \"Simulation finished\""));
    assert!(tb.contains("wait;"));
}

#[test]
fn test_counter_tb_library_declarations() {
    let tbs = gen_tb_from_file("testdata/counter.vhd");
    let tb = &tbs[0];
    assert!(tb.contains("library ieee;"));
    assert!(tb.contains("use ieee.std_logic_1164.all;"));
    assert!(tb.contains("use ieee.numeric_std.all;"));
}

// === uart_tx.vhd テスト ===

#[test]
fn test_uart_tb_entity_name() {
    let tbs = gen_tb_from_file("testdata/uart_tx.vhd");
    assert_eq!(tbs.len(), 1);
    assert!(tbs[0].contains("entity uart_tx_tb is"));
}

#[test]
fn test_uart_tb_all_ports_as_signals() {
    let tbs = gen_tb_from_file("testdata/uart_tx.vhd");
    let tb = &tbs[0];
    assert!(tb.contains("signal clk : std_logic"));
    assert!(tb.contains("signal reset : std_logic"));
    assert!(tb.contains("signal tx_start : std_logic"));
    assert!(tb.contains("signal tx_data : std_logic_vector(7 downto 0)"));
    assert!(tb.contains("signal tx_out : std_logic"));
    assert!(tb.contains("signal tx_busy : std_logic"));
    assert!(tb.contains("signal tx_done : std_logic"));
}

#[test]
fn test_uart_tb_component_ports() {
    let tbs = gen_tb_from_file("testdata/uart_tx.vhd");
    let tb = &tbs[0];
    assert!(tb.contains("component uart_tx is"));
    assert!(tb.contains("tx_start : in std_logic"));
    assert!(tb.contains("tx_data : in std_logic_vector(7 downto 0)"));
    assert!(tb.contains("tx_out : out std_logic"));
    assert!(tb.contains("tx_busy : out std_logic"));
    assert!(tb.contains("tx_done : out std_logic"));
}

#[test]
fn test_uart_tb_clock_and_reset() {
    let tbs = gen_tb_from_file("testdata/uart_tx.vhd");
    let tb = &tbs[0];
    assert!(tb.contains("clk_process: process"));
    assert!(tb.contains("reset <= '1';"));
    assert!(tb.contains("reset <= '0';"));
}

#[test]
fn test_uart_tb_input_hints() {
    let tbs = gen_tb_from_file("testdata/uart_tx.vhd");
    let tb = &tbs[0];
    // clk, reset以外の入力ポートがコメントで示される
    assert!(tb.contains("-- tx_start <= '0';"));
    assert!(tb.contains("-- tx_data <= (others => '0');"));
}

// === alu.vhd テスト ===

#[test]
fn test_alu_tb_entity_name() {
    let tbs = gen_tb_from_file("testdata/alu.vhd");
    assert_eq!(tbs.len(), 1);
    assert!(tbs[0].contains("entity alu_tb is"));
}

#[test]
fn test_alu_tb_all_ports_as_signals() {
    let tbs = gen_tb_from_file("testdata/alu.vhd");
    let tb = &tbs[0];
    assert!(tb.contains("signal op_a : std_logic_vector(3 downto 0)"));
    assert!(tb.contains("signal op_b : std_logic_vector(3 downto 0)"));
    assert!(tb.contains("signal alu_op : std_logic_vector(2 downto 0)"));
    assert!(tb.contains("signal result : std_logic_vector(3 downto 0)"));
    assert!(tb.contains("signal carry_out : std_logic"));
    assert!(tb.contains("signal zero_flag : std_logic"));
}

#[test]
fn test_alu_tb_port_map() {
    let tbs = gen_tb_from_file("testdata/alu.vhd");
    let tb = &tbs[0];
    assert!(tb.contains("uut: alu"));
    assert!(tb.contains("op_a => op_a"));
    assert!(tb.contains("op_b => op_b"));
    assert!(tb.contains("alu_op => alu_op"));
    assert!(tb.contains("result => result"));
    assert!(tb.contains("carry_out => carry_out"));
    assert!(tb.contains("zero_flag => zero_flag"));
}

#[test]
fn test_alu_tb_input_hints() {
    let tbs = gen_tb_from_file("testdata/alu.vhd");
    let tb = &tbs[0];
    // clk, reset以外の入力ポートがコメントで示される
    assert!(tb.contains("-- op_a <= (others => '0');"));
    assert!(tb.contains("-- op_b <= (others => '0');"));
    assert!(tb.contains("-- alu_op <= (others => '0');"));
}

// === カスタム設定テスト ===

#[test]
fn test_custom_clock_period() {
    let source = fs::read_to_string("testdata/counter.vhd").unwrap();
    let result = analyze_vhdl(&source).unwrap();
    let config = TbConfig {
        clock_period_ns: 20,
    };
    let tb = generate_testbench(&result.entities[0], &config);
    // 周期20ns -> 半周期10ns
    assert!(tb.contains("wait for 10 ns;"));
    assert!(tb.contains("周期 20 ns"));
}

// === ポートなしentityテスト ===

#[test]
fn test_empty_entity_tb() {
    let source = "entity empty is\nend entity empty;";
    let result = analyze_vhdl(source).unwrap();
    let config = TbConfig::default();
    let tb = generate_testbench(&result.entities[0], &config);
    assert!(tb.contains("entity empty_tb is"));
    // クロック・リセットがないのでそれらのプロセスがない
    assert!(!tb.contains("clk_process"));
    assert!(!tb.contains("reset <="));
}
