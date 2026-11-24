// MPLab C++ Example 5: Binding, static vs dynamic — shows WHEN the compiler
// ties a function call to a function body. Non-virtual calls are bound
// statically at compile time by the static (declared) type; virtual calls are
// bound dynamically at run time through the object's vtable by its dynamic
// (actual) type. One base pointer produces two different behaviours.
//
// Reads nothing.

#include <iostream>

class Animal {
public:
    void speak_nonvirtual() {
        std::cout << "  Animal::speak_nonvirtual (bound by declared type)\n";
    }
    virtual void speak_virtual() {
        std::cout << "  Animal::speak_virtual\n";
    }
    virtual ~Animal() = default;  // virtual dtor: deleting via base ptr is safe
};

class Dog : public Animal {
public:
    // redefinition, NOT an override of virtual dispatch — hides the base fn
    void speak_nonvirtual() {
        std::cout << "  Dog::speak_nonvirtual (never reached via Animal*)\n";
    }
    void speak_virtual() override {  // override keyword asks the compiler to
        std::cout << "  Dog::speak_virtual (reached via the vtable)\n";
    }                                // verify a base virtual really is overridden
};

int main() {
    std::cout << "== Same object, same pointer, two binding rules ==\n";
    Dog d;
    Animal* a = &d;              // declared type: Animal; actual object: Dog

    std::cout << "non-virtual call (static binding, at compile time):\n";
    a->speak_nonvirtual();       // compiler sees Animal* -> Animal's version.
                                 // Dog::speak_nonvirtual is IGNORED: it only
                                 // hides the name for Dog-typed expressions.

    std::cout << "virtual call (dynamic binding, at run time):\n";
    a->speak_virtual();          // compiler emits a vtable lookup: Dog's
                                 // override runs, chosen by the object's
                                 // actual type at run time.

    std::cout << "\n== vtable mechanics (in comments) ==\n";
    // Each class with virtual functions has a hidden vtable: an array of
    // function pointers. The compiler puts a vptr into every such object;
    // a virtual call becomes ptr->vptr[slot]() instead of a fixed call.
    // Non-virtual calls pay nothing; virtual calls pay one indirection.
    Dog* d_ptr = &d;
    std::cout << "through Dog*, Dog::speak_nonvirtual now runs:\n";
    d_ptr->speak_nonvirtual();   // declared type Dog -> Dog's version this time
    return 0;
}
