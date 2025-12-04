# Crystal palace rust development kit 
This crate provides an easy way to develop [PICOs](https://tradecraftgarden.org/docs.html#picos) in rust using macros and helper functions to make your life as a developer easier.
 
[The Crystal Palace linker](https://tradecraftgarden.org/crystalpalace.html) looks for certain instruction patterns that are originally emitted by the MinGW compiler. To get the linker working in rust it is required that the final object binary also has these instruction patterns.

To fulfill these requirements, inline assembly is used to emulate the MinGW compilers output and prevent specific optimisations from rustc and llvm. This has been packaged inside macros that abstract away the direct assembly and provide a clean, easy to use interface.

Examples of usage can be found [here](https://github.com/laachy/tradecraft-garden-rs)


# Usage
### Adding the crate to your project

   ```powershell
   cargo add crystal-sdk
   ```
   
### Developing with crystal-sdk

 - **Importing WinAPI functions**

For raw symbols to look like __imp_MOD$Func and  __imp_Func respectively:

    import!(KERNEL32!VirtualAlloc(lpAddress:  LPVOID, dwSize: usize, flAllocationType:  DWORD, flProtect:  DWORD) ->  LPVOID);` 

    import!(LoadLibraryA(arg1:  LPCSTR) ->  HMODULE);

Use the imported function as if it were a normal function. To get the function pointer, the current way to do so is to append _ptr()

    GetProcAddress_ptr()


 - **Retrieving appended data**

To append data to "my_data" and retrieve it:

    append_data!(my_data, findAppendedDLL);

    let dll = findAppendedDLL();

 - **Retrieving resources**

Resources have an appended length meaning we can use byte slices that reference the exact memory location. You can use:

`get_resource(findAppendedDLL())`


 - **Using builtin memory functions**

Functions such as memcpy, memset, memmove, and strlen are inlined and can be used by exposing the `mem` module from within `crystal-sdk`
 
#### Other notes on PICO rust development
 - To prevent optimisations, globals should be declared as mutable if they will be patched at link time
 - Any function that will be called externally from another PICO should be declared as extern "C" with no_mangle as such:

   `#[unsafe(no_mangle)] extern  "C"  fn  go()`

- To comply with automatic generated bindings, system functions must be defined as extern "C". While "system" is the correct ABI, on x86-64 "C" and "system" are the exact same. This was the easiest solution as bindgen does not generate "system"



# TODO and issues

 1. Create a better interface for globals and patching
 2. Make the import function interface and usage cleaner by implementing traits
 3. Add x86 support

# Credits
Helped me with symbol names and might use the deref method later: https://github.com/wumb0/rust_bof/blob/master/libs/bofhelper/src/lib.rs

