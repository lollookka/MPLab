// MPLab C++ Example 3: Static type checking — shows what the compiler checks
// BEFORE the program runs. Uncommenting the two broken lines below makes g++
// reject the whole file with the errors shown in the comments; that rejection
// IS the type system working. The rest demonstrates const-correctness and
// compile-time type deduction with auto.
//
// Reads nothing.

#include <iostream>
#include <typeinfo>

int main() {
    // == What the compiler REJECTS (uncomment either line to see the error) ==
    //
    // int x = "hello";
    //   error: invalid conversion from 'const char*' to 'int'
    //     [-fpermissive] — a pointer is not an integer, caught at compile time.
    //
    // y = 5;
    //   error: 'y' was not declared in this scope — no undeclared names, ever.

    std::cout << "== const-correctness ==\n";
    const int limit = 100;          // const promises: this will never change
    const int* p = &limit;          // pointer-to-const: can read, cannot write
    std::cout << "limit = " << *p << " (readable through const int*)\n";
    // *p = 5;
    //   error: assignment of read-only location '* p' — the promise is enforced.
    std::cout << "writing through a const pointer would not compile (see comment)\n";

    std::cout << "\n== auto: the compiler still knows every type ==\n";
    auto whole = 42;                // deduced: int
    auto fraction = 4.5;            // deduced: double
    auto letter = 'c';              // deduced: char
    auto text = std::string("hi");  // deduced: std::string
    // auto must have an initializer; 'auto blank;' would not compile.
    std::cout << "auto 42    -> " << typeid(whole).name() << "\n";
    std::cout << "auto 4.5   -> " << typeid(fraction).name() << "\n";
    std::cout << "auto 'c'   -> " << typeid(letter).name() << "\n";
    std::cout << "auto string-> " << typeid(text).name() << "\n";

    std::cout << "\n== mismatched types still mix safely only via explicit casts ==\n";
    std::cout << "int(" << whole << ") + double(" << fraction
              << ") = " << static_cast<double>(whole) + fraction << "\n";
    return 0;
}
