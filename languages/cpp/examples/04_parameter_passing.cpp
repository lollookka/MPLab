// MPLab C++ Example 4: Parameter passing — shows the three ways arguments
// reach a function in C++: by value (a copy, changes are invisible outside),
// by reference (an alias, changes are visible), and by const reference
// (read-only alias, the cheap way to pass big objects). Ends with std::move
// handing ownership of a string's buffer over without copying it.
//
// Reads nothing.

#include <iostream>
#include <string>
#include <utility>

void by_value(std::string s) {           // s is a fresh COPY of the argument
    s += " (modified in callee)";
    std::cout << "  by_value  sees: " << s << "\n";
}

void by_reference(std::string& s) {      // s IS the caller's object, another name
    s += " (modified in callee)";
    std::cout << "  by_ref    sees: " << s << "\n";
}

void by_const_ref(const std::string& s) { // read-only alias, no copy made
    std::cout << "  const_ref sees: " << s << "  (would not compile to modify)\n";
    // s += "x";  // error: binding reference of type 'std::string&' to value of
    //            // type 'const std::string' discards qualifiers
}

int main() {
    std::string original = "hello";
    std::cout << "original before: \"" << original << "\"\n\n";

    by_value(original);
    std::cout << "after by_value : \"" << original
              << "\"  (copy was changed, original untouched)\n";

    by_reference(original);
    std::cout << "after by_ref   : \"" << original
              << "\"  (callee modified the caller's object)\n";

    by_const_ref(original);
    std::cout << "after const_ref: \"" << original << "\"\n";

    std::cout << "\n== std::move: move semantics ==\n";
    std::string big(10, 'x');            // a string holding 10 characters
    std::string stolen = std::move(big); // buffer handed over, no copy of chars
    std::cout << "moved-from size: " << big.size()
              << "  moved-to size: " << stolen.size() << "\n";
    std::cout << "big is now empty/unspecified; stolen owns the characters\n";
    return 0;
}
