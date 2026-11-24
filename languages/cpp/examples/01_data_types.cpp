// MPLab C++ Example 1: Static typing & data types — shows how every value has a
// fixed type at compile time, how big each built-in type is on this machine,
// and what happens when one type is narrowed into another.
//
// Reads ONE integer from stdin (type it in MPLab's input box, e.g. 42).

#include <iostream>
#include <limits>

int main() {
    std::cout << "== Sizes (sizeof, in bytes) ==\n";
    std::cout << "int    : " << sizeof(int)    << " bytes, range "
              << std::numeric_limits<int>::min() << " .. "
              << std::numeric_limits<int>::max() << "\n";
    std::cout << "double : " << sizeof(double)  << " bytes\n";
    std::cout << "char   : " << sizeof(char)    << " byte\n";
    std::cout << "bool   : " << sizeof(bool)    << " byte (only true/false)\n";

    std::cout << "\n== Integer vs floating division ==\n";
    std::cout << "7 / 2      = " << 7 / 2 << "          (int / int -> int, remainder dropped)\n";
    std::cout << "7.0 / 2.0  = " << 7.0 / 2.0 << "      (double / double -> double)\n";
    std::cout << "7 % 2      = " << 7 % 2 << "          (% is the remainder, integers only)\n";

    std::cout << "\n== Implicit narrowing ==\n";
    double pi = 3.14;
    int truncated = pi;              // compiles, but the .14 is silently cut off
    std::cout << "double 3.14 -> int : " << truncated << " (fractional part lost, no error)\n";

    std::cout << "\n== Your input, pinned to its type ==\n";
    int n;                           // type decided at compile time, never changes
    std::cin >> n;                   // reads one whitespace-delimited integer
    std::cout << "You typed: " << n << "\n";
    std::cout << "It fits in 'int', whose range is "
              << std::numeric_limits<int>::min() << " .. "
              << std::numeric_limits<int>::max() << "\n";
    std::cout << "Twice it: " << n + n << "\n";
    return 0;
}
