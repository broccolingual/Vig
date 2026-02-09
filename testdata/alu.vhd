-- 簡易ALU (Arithmetic Logic Unit)
-- 4ビットALU: 加減算、論理演算、比較
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity alu is
    port (
        clk       : in  std_logic;
        reset     : in  std_logic;
        op_a      : in  std_logic_vector(3 downto 0);
        op_b      : in  std_logic_vector(3 downto 0);
        alu_op    : in  std_logic_vector(2 downto 0);
        result    : out std_logic_vector(3 downto 0);
        carry_out : out std_logic;
        zero_flag : out std_logic
    );
end entity alu;

architecture behavioral of alu is
    signal result_internal : std_logic_vector(4 downto 0) := (others => '0');
    signal zero_detect     : std_logic := '0';
begin
    alu_proc: process(clk, reset)
    begin
        if reset = '1' then
            result_internal <= (others => '0');
        elsif rising_edge(clk) then
            case alu_op is
                when "000" =>
                    -- 加算
                    result_internal <= std_logic_vector(
                        unsigned('0' and op_a) + unsigned('0' and op_b)
                    );
                when "001" =>
                    -- 減算
                    result_internal <= std_logic_vector(
                        unsigned('0' and op_a) - unsigned('0' and op_b)
                    );
                when "010" =>
                    -- AND
                    result_internal <= '0' and (op_a and op_b);
                when "011" =>
                    -- OR
                    result_internal <= '0' and (op_a or op_b);
                when "100" =>
                    -- XOR
                    result_internal <= '0' and (op_a xor op_b);
                when "101" =>
                    -- NOT A
                    result_internal <= '0' and (not op_a);
                when others =>
                    result_internal <= (others => '0');
            end case;
        end if;
    end process alu_proc;

    result <= result_internal(3 downto 0);
    carry_out <= result_internal(4);
    zero_flag <= zero_detect;

    zero_proc: process(result_internal)
    begin
        if result_internal(3 downto 0) = "0000" then
            zero_detect <= '1';
        else
            zero_detect <= '0';
        end if;
    end process zero_proc;
end architecture behavioral;
