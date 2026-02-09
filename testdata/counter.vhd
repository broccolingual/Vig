-- 簡単なカウンタの例
library ieee;
use ieee.std_logic_1164.all;

entity counter is
    port (
        clk   : in  std_logic;
        reset : in  std_logic;
        count : out std_logic_vector(7 downto 0)
    );
end entity counter;

architecture behavioral of counter is
    signal counter_value : integer := 0;
begin
    process(clk, reset)
    begin
        if reset = '1' then
            counter_value <= 0;
        elsif rising_edge(clk) then
            counter_value <= counter_value + 1;
        end if;
    end process;

    count <= std_logic_vector(to_unsigned(counter_value, 8));
end architecture behavioral;
