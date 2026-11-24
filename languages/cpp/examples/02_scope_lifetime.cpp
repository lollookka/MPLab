// MPLab C++ Example 2: Scope, lifetime & RAII — shows how a variable's block
// decides when it exists, how shadowing hides an outer name inside a nested
// block, and how RAII ties a resource's lifetime to an object so the
// destructor always releases it — automatically, even on early return.
//
// Reads ONE line from stdin and uses it as a "resource name"
// (e.g. type: db-connection)

#include <iostream>
#include <string>

class Resource {  // RAII: acquire in the constructor, release in the destructor
public:
    explicit Resource(std::string name) : name_(std::move(name)) {
        std::cout << "  [acquire] resource '" << name_ << "' opened\n";
    }
    ~Resource() {
        std::cout << "  [release] resource '" << name_ << "' closed\n";
    }
private:
    std::string name_;
};

int main() {
    std::string resource_name;
    std::getline(std::cin, resource_name);   // one whole line, spaces allowed

    std::string x = "outer";
    std::cout << "outside block, x = " << x << "\n";
    {
        std::string x = "inner";             // SHADOWS the outer x until this block ends
        std::cout << "inside  block, x = " << x << " (this x shadows the outer one)\n";
    }
    std::cout << "back outside, x = " << x << " (inner x is destroyed, scope ended)\n";

    std::cout << "\n== Stack vs heap lifetime ==\n";
    {
        int stack_var = 1;                   // exists only inside these braces
        int* heap_var = new int(2);          // exists until someone calls delete
        std::cout << "stack value: " << stack_var
                  << ", heap value: " << *heap_var << "\n";
        delete heap_var;                     // heap memory needs an explicit delete
        std::cout << "heap object deleted explicitly; stack object ends at brace\n";
    }                                        // <- stack_var destroyed HERE, silently

    std::cout << "\n== RAII: construction/destruction order ==\n";
    std::cout << "entering RAII scope\n";
    {
        Resource r1(resource_name);          // ctor prints [acquire]
        Resource r2("spare-conn");           // nested: r2 is built after r1...
        std::cout << "  working with both resources\n";
    }                                        // ...and destroyed BEFORE r1 (reverse order)
    std::cout << "left RAII scope\n";
    return 0;                                // main's locals destroyed here too
}
