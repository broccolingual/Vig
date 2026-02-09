-- UART送信モジュール
-- ボーレート設定可能な非同期シリアル送信器
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity uart_tx is
    port (
        clk       : in  std_logic;
        reset     : in  std_logic;
        tx_start  : in  std_logic;
        tx_data   : in  std_logic_vector(7 downto 0);
        tx_out    : out std_logic;
        tx_busy   : out std_logic;
        tx_done   : out std_logic
    );
end entity uart_tx;

architecture rtl of uart_tx is
    -- ボーレート分周カウンタ (115200bps @ 50MHz -> 434クロック)
    signal baud_counter : integer := 0;
    signal baud_tick    : std_logic := '0';

    -- ビットカウンタ (start + 8data + stop = 10ビット)
    signal bit_index : integer := 0;

    -- 送信シフトレジスタ
    signal shift_reg : std_logic_vector(9 downto 0) := (others => '1');

    -- 状態管理
    signal transmitting : std_logic := '0';
    signal done_flag    : std_logic := '0';
begin
    -- ボーレートジェネレータ
    baud_gen: process(clk, reset)
    begin
        if reset = '1' then
            baud_counter <= 0;
            baud_tick <= '0';
        elsif rising_edge(clk) then
            if transmitting = '1' then
                if baud_counter = 433 then
                    baud_counter <= 0;
                    baud_tick <= '1';
                else
                    baud_counter <= baud_counter + 1;
                    baud_tick <= '0';
                end if;
            else
                baud_counter <= 0;
                baud_tick <= '0';
            end if;
        end if;
    end process baud_gen;

    -- 送信制御
    tx_ctrl: process(clk, reset)
    begin
        if reset = '1' then
            shift_reg <= (others => '1');
            bit_index <= 0;
            transmitting <= '0';
            done_flag <= '0';
        elsif rising_edge(clk) then
            done_flag <= '0';

            if transmitting = '0' then
                if tx_start = '1' then
                    -- スタートビット(0) + データ + ストップビット(1)
                    shift_reg <= '1' and tx_data and '0';
                    bit_index <= 0;
                    transmitting <= '1';
                end if;
            else
                if baud_tick = '1' then
                    if bit_index = 9 then
                        transmitting <= '0';
                        done_flag <= '1';
                    else
                        shift_reg <= '1' and shift_reg(9 downto 1);
                        bit_index <= bit_index + 1;
                    end if;
                end if;
            end if;
        end if;
    end process tx_ctrl;

    tx_out <= shift_reg(0);
    tx_busy <= transmitting;
    tx_done <= done_flag;
end architecture rtl;
