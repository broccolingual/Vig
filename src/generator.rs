use crate::analyzer::{EntityDef, PortDef, PortDirection, VhdlType};

/// テストベンチ生成の設定
pub struct TbConfig {
    /// クロック周期（ns）
    pub clock_period_ns: u64,
}

impl Default for TbConfig {
    fn default() -> Self {
        Self {
            clock_period_ns: 10,
        }
    }
}

/// EntityDefからテストベンチのVHDLコードを生成する
pub fn generate_testbench(entity: &EntityDef, config: &TbConfig) -> String {
    let tb_name = format!("{}_tb", entity.name);
    let clk_port = find_clock_port(&entity.ports);
    let rst_port = find_reset_port(&entity.ports);

    let mut out = String::new();

    // ライブラリ宣言
    out.push_str("library ieee;\n");
    out.push_str("use ieee.std_logic_1164.all;\n");
    out.push_str("use ieee.numeric_std.all;\n");
    out.push('\n');

    // テストベンチentity（ポートなし）
    out.push_str(&format!("entity {} is\n", tb_name));
    out.push_str(&format!("end entity {};\n", tb_name));
    out.push('\n');

    // architecture
    out.push_str(&format!("architecture testbench of {} is\n", tb_name));
    out.push('\n');

    // コンポーネント宣言
    out.push_str(&gen_component(entity));
    out.push('\n');

    // 信号宣言
    out.push_str(&gen_signals(&entity.ports));
    out.push('\n');

    out.push_str("begin\n");
    out.push('\n');

    // DUTインスタンス
    out.push_str(&gen_dut_instance(entity));
    out.push('\n');

    // クロック生成プロセス
    if let Some(clk) = &clk_port {
        out.push_str(&gen_clock_process(clk, config.clock_period_ns));
        out.push('\n');
    }

    // スティミュラスプロセス
    out.push_str(&gen_stimulus_process(
        &entity.ports,
        clk_port.as_deref(),
        rst_port.as_deref(),
        config.clock_period_ns,
    ));
    out.push('\n');

    out.push_str("end architecture testbench;\n");
    out
}

/// 型のVHDL文字列表現
fn type_to_vhdl(vhdl_type: &VhdlType) -> String {
    match vhdl_type {
        VhdlType::StdLogic => "std_logic".to_string(),
        VhdlType::StdLogicVector { high, low } => {
            format!("std_logic_vector({} downto {})", high, low)
        }
        VhdlType::Integer => "integer".to_string(),
        VhdlType::Boolean => "boolean".to_string(),
        VhdlType::Other(name) => name.clone(),
    }
}

/// 型のデフォルト初期値
fn type_default_value(vhdl_type: &VhdlType) -> String {
    match vhdl_type {
        VhdlType::StdLogic => "'0'".to_string(),
        VhdlType::StdLogicVector { .. } => "(others => '0')".to_string(),
        VhdlType::Integer => "0".to_string(),
        VhdlType::Boolean => "false".to_string(),
        VhdlType::Other(_) => "'0'".to_string(),
    }
}

/// clk を含むポートを探す（大文字小文字を区別しない）
fn find_clock_port(ports: &[PortDef]) -> Option<String> {
    let lower_contains = |name: &str, pat: &str| name.to_lowercase().contains(pat);
    ports
        .iter()
        .find(|p| p.direction == PortDirection::In && lower_contains(&p.name, "clk"))
        .map(|p| p.name.clone())
}

/// reset を含むポートを探す（大文字小文字を区別しない）
fn find_reset_port(ports: &[PortDef]) -> Option<String> {
    let lower_contains = |name: &str, pat: &str| name.to_lowercase().contains(pat);
    ports
        .iter()
        .find(|p| {
            p.direction == PortDirection::In
                && (lower_contains(&p.name, "rst") || lower_contains(&p.name, "reset"))
        })
        .map(|p| p.name.clone())
}

/// コンポーネント宣言を生成
fn gen_component(entity: &EntityDef) -> String {
    let mut s = String::new();
    s.push_str(&format!("    component {} is\n", entity.name));
    if !entity.ports.is_empty() {
        s.push_str("        port (\n");
        for (i, port) in entity.ports.iter().enumerate() {
            let dir = match port.direction {
                PortDirection::In => "in",
                PortDirection::Out => "out",
                PortDirection::Inout => "inout",
                PortDirection::Buffer => "buffer",
            };
            let sep = if i + 1 < entity.ports.len() { ";" } else { "" };
            s.push_str(&format!(
                "            {} : {} {}{}",
                port.name,
                dir,
                type_to_vhdl(&port.vhdl_type),
                sep
            ));
            s.push('\n');
        }
        s.push_str("        );\n");
    }
    s.push_str(&format!("    end component {};\n", entity.name));
    s
}

/// ポートに対応する信号宣言を生成
fn gen_signals(ports: &[PortDef]) -> String {
    let mut s = String::new();
    for port in ports {
        s.push_str(&format!(
            "    signal {} : {} := {};\n",
            port.name,
            type_to_vhdl(&port.vhdl_type),
            type_default_value(&port.vhdl_type)
        ));
    }
    s
}

/// DUTインスタンスを生成
fn gen_dut_instance(entity: &EntityDef) -> String {
    let mut s = String::new();
    s.push_str(&format!("    uut: {}\n", entity.name));
    if !entity.ports.is_empty() {
        s.push_str("        port map (\n");
        for (i, port) in entity.ports.iter().enumerate() {
            let sep = if i + 1 < entity.ports.len() { "," } else { "" };
            s.push_str(&format!(
                "            {} => {}{}",
                port.name, port.name, sep
            ));
            s.push('\n');
        }
        s.push_str("        );\n");
    } else {
        s.push_str("    ;\n");
    }
    s
}

/// クロック生成プロセスを生成
fn gen_clock_process(clk_name: &str, period_ns: u64) -> String {
    let half = period_ns / 2;
    let mut s = String::new();
    s.push_str(&format!("    -- クロック生成 (周期 {} ns)\n", period_ns));
    s.push_str("    clk_process: process\n");
    s.push_str("    begin\n");
    s.push_str(&format!("        {} <= '0';\n", clk_name));
    s.push_str(&format!("        wait for {} ns;\n", half));
    s.push_str(&format!("        {} <= '1';\n", clk_name));
    s.push_str(&format!("        wait for {} ns;\n", half));
    s.push_str("    end process clk_process;\n");
    s
}

/// スティミュラスプロセスを生成
fn gen_stimulus_process(
    ports: &[PortDef],
    clk_name: Option<&str>,
    rst_name: Option<&str>,
    period_ns: u64,
) -> String {
    let mut s = String::new();
    s.push_str("    -- テストシナリオ\n");
    s.push_str("    stim_process: process\n");
    s.push_str("    begin\n");

    // リセットシーケンス
    if let Some(rst) = rst_name {
        s.push_str("        -- リセット\n");
        s.push_str(&format!("        {} <= '1';\n", rst));
        s.push_str(&format!("        wait for {} ns;\n", period_ns * 2));
        s.push_str(&format!("        {} <= '0';\n", rst));
        s.push_str(&format!("        wait for {} ns;\n", period_ns * 2));
        s.push('\n');
    }

    s.push_str("        -- TODO: テストパターンを記述\n");
    s.push_str(&format!("        wait for {} ns;\n", period_ns * 10));
    s.push('\n');

    // 入力ポートの初期化例をコメントで示す
    let input_ports: Vec<&PortDef> = ports
        .iter()
        .filter(|p| {
            (p.direction == PortDirection::In || p.direction == PortDirection::Inout)
                && Some(p.name.as_str()) != clk_name
                && Some(p.name.as_str()) != rst_name
        })
        .collect();

    if !input_ports.is_empty() {
        s.push_str("        -- 入力信号の例:\n");
        for port in &input_ports {
            s.push_str(&format!(
                "        -- {} <= {};\n",
                port.name,
                type_default_value(&port.vhdl_type)
            ));
        }
        s.push_str(&format!("        -- wait for {} ns;\n", period_ns));
        s.push('\n');
    }

    s.push_str("        -- シミュレーション終了\n");
    s.push_str("        assert false report \"Simulation finished\" severity note;\n");
    s.push_str("        wait;\n");
    s.push_str("    end process stim_process;\n");
    s
}
