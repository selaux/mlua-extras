# MLua Extras

> [!NOTE]
> Feel free to use this crate and start working with ideas and features that could be useful.
>
> Pull requests and contribution are encouraged
>
> If you want to discuss this project, you can do that [here](https://github.com/Tired-Fox/mlua-extras/discussions/1)

___

The goal of this project is to add a light convenience layer wrapping the [`mlua`](https://docs.rs/mlua/latest/mlua/) crate. The goal isn't to change the way that `mlua` is used, but instead to make `lua` embedded development in `Rust` more enjoyable.

## Similar Projects

- `Tealr`: A project to enhance and extend `mlua` with a focus in type information and documentation along with a type syntax in the lua code itself with the `tealr` syntax.
    - This crate is a great choice if you need: type syntax, type information, documentation generation

## Features

- Helper Traits
    - `Require`
        - call `require` which allows for a lua style call to get data from the lua engine.
        - ex: `table.require::<String>("nested.tables.name")?` == `local name = require('nested.tables').name` 
    - `Module`
        - Syntax and API mimics `UserData` adding fields, functions and methods. Instead of createing a lua `userdata` type, it adds these values to a lua `table`.
        - This allows for all the data to be held in lua
        - This allows for a complex lua api without creating a bunch of userdata objects
    - `LuaExtras`
        - Manipulate the lua [`path`](https://www.lua.org/manual/5.1/manual.html#pdf-package.path) and [`cpath`](https://www.lua.org/manual/5.1/manual.html#pdf-package.cpath) variables with `append`, `prepend`, and `set` methods for each variant. It also includes the ability to add multiple paths with each variant.
        - Set global variables and functions with `set_global("value", "value")` and `set_global_function("func", |lua, ()| Ok(()))` which wold replace `lua.globals().set("value", "value)` and `lua.globals().set("func", lua.create_function(|lua, ()| Ok(()))?)` respectively

- Typed Lua Traits
    - `Typed`
        - Generate a `Type` and `Param` for a rust type so it can be used both as a type and as a parameter for a function
    - `TypedUserData`
        - Typed variant of [`mlua::UserData`](https://docs.rs/mlua/latest/mlua/trait.UserData.html) with an additional `add_documentation` method to add doc comments to the [`UserData`](https://docs.rs/mlua/latest/mlua/trait.UserData.html) type
        - An extra `document` method is added to the `TypedDataFields` and `TypedDataMethods` for [`add_fields`](https://docs.rs/mlua/latest/mlua/trait.UserData.html#method.add_fields) and [`add_methods`](https://docs.rs/mlua/latest/mlua/trait.UserData.html#method.add_methods). This will queue doc comments to be added to the next field or method that is added.
        - All types from function parameters and and return types are stored for fields, functions, and methods.
        - This trait is mainly used when generating type definitions. If it is called through the [`UserData`](https://docs.rs/mlua/latest/mlua/trait.UserData.html) derive macro it will ignore all types and documentation
    - `TypedDataFields`: Implemented on a generator for `TypedUserData` ([`add_fields`](https://docs.rs/mlua/latest/mlua/trait.UserData.html#method.add_fields))
    - `TypedDataMethods`: Implemented on a generator for `TypedUserData` ([`add_methods`](https://docs.rs/mlua/latest/mlua/trait.UserData.html#method.add_methods))
    - `TypedDataDocumentation`: Implemented on a generator for `TypedUserData` (`add_documentation`)

- Derive Macros
    - `Typed`: Auto implement the `Typed` trait to get type information for both `struct` and `enum`
    - `UserData`: Auto implement the [`mlua::UserData`](https://docs.rs/mlua/latest/mlua/trait.UserData.html) trait for rust types that also implement `TypedUserData`. This will pass through the [`UserData`](https://docs.rs/mlua/latest/mlua/trait.UserData.html) [`add_methods`](https://docs.rs/mlua/latest/mlua/trait.UserData.html#method.add_methods) and [`add_fields`](https://docs.rs/mlua/latest/mlua/trait.UserData.html#method.add_fields) to the `TypedUserData`'s version. This will ignore all documentation and types.

- Macros
    - `function`: Write lua functions more like Rust's syntax

    Instead of this:

    ```rust
    lua.create_function(|lua, ()| Ok(())) 
    ```

    You can now write this:

    ```rust
    function!{
        lua fn name(lua) {
            Ok(())
        }
    }
    ```

    The difference isn't huge, but it could make the syntax more readable.

    > [!NOTE]
    > This also helps with assigning functions to nested tables.

    > [!WARNING]
    > This requires the `LuaExtras` trait when adding functions to nested tables with `lua` as the starting point. This requires the `Require` trait when starting from any other table.

    ```rust
    lua.require::<Table>("nested.fn").set("name", lua.create_function(|lua, ()| Ok(())));

    // vs

    function! {
        lua fn lua::nested::fn.name(lua) {
            Ok(())
        }
    }
    ```

## Ideas and Planned Features

- Fully featured definition file generation
- Fully featured documentation generation
- Fully featured addon generator when creating a lua modules with `mlua`'s `module` feature
- Better and more informative type errors associated with lua type definitions and output generation
- More expressive way of defining exposed lua api types
    - Generic types
    - Doc comments for params and return types

## References

- [`lua`](https://www.lua.org/)
- [`mlua`](https://github.com/mlua-rs/mlua)
- [`Tealr`](https://github.com/lenscas/tealr)
- [`Luau`](https://luau.org/)
- [`Lua Language Server`](https://github.com/LuaLS/lua-language-server)

## Example Syntax

**Helpers**

```rust
use mlua::{Lua, Table, Function, Variadic, Value};

fn main() -> mlua::Result<()> {
    let lua = Lua::new();

    // Prepend path to the lua `path`
    let path = lua.globals().get::<Table>("package")?.get::<String>("path");
    lua.globals().get::<Table>("package")?.set("path", format!("?.lua;{path}"))?;

    let temp = lua.create_table()?;
    temp.set("getName", lua.create_function(|lua, ()| Ok("name"))?;

    // Get a nested function: `table.unpack`
    let unpack = lua.globals().get::<Table>("table")?.get::<_, Function>("unpack")?;
    // Call the `table.unpack` function
    let _ = unpack.call::<Variadic<Value>>(temp)?;
    Ok(())
}
```

```rust
use mlua_extras::{
    mlua::{self, Lua, Table, Variadic, Value}
    extras::{Require, LuaExtras},
    typed::TypedFunction,
    function,
};

fn main() -> mlua::Result<()> {
    let lua = Lua::new();

    // Prepend path to the lua `path`
    lua.prepend_path("?.lua")?;

    let temp = lua.create_table()?;
    function! {
        lua fn temp.name(lua) {
            Ok("name")
        }
    }

    // Get a nested function: `table.unpack`
    let unpack = lua.require::<TypedFunction<Table, Variadic<Value>>>("table.unpack")?;
    // Call the `table.unpack` function
    let _ = unpack.call(temp)?;
    Ok(())
}
```

**Types**

```rust
use serde::Deserialize;
use mlua_extras::{
    mlua::{self, Lua, Table, Variadic, Value},
    extras::{ Require, LuaExtras },
    typed::{
        generator::{Definition, Definitions, DefinitionFileGenerator},
        TypedFunction, TypedUserData
    },
    Typed, UserData, function,
};

#[derive(Default, Debug, Clone, Copy, Typed, Deserialize)]
enum SystemColor {
    #[default]
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Cyan,
    Magenta,
    White,
}

#[derive(Debug, Clone, Copy, Typed, UserData, Deserialize)]
#[serde(untagged)]
enum Color {
    System(SystemColor),
    Xterm(u8),
    Rgb(u8, u8, u8),
}
impl Default for Color {
    fn default() -> Self {
        Color::System(SystemColor::default())
    }
}
impl<'lua> FromLua<'lua> for Color {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> mlua::prelude::LuaResult<Self> {
        match value {
            Value::UserData(data) => data.borrow::<Self>().map(|v| *v),
            // Use serde deserialize if not userdata
            other => lua.from_value(other),
        }
    }
}

#[derive(Debug, Clone, Copy, Typed, UserData, Deserialize)]
struct Example {
    color: Color
}
impl TypedUserData for Example {
    fn add_documentation<F: mlua_extras::typed::TypedDataDocumentation<Self>>(docs: &mut F) {
        docs.add("This is a doc comment section for the overall type");
    }

    fn add_fields<'lua, F: TypedDataFields<'lua, Self>>(fields: &mut F) {
        fields
            .document("Example complex type")
            .add_field_method_get_set(
                "color",
                |_lua, this| Ok(this.color),
                |_lua, this, clr: Color| {
                    this.color = clr;
                    Ok(())
                },
            );
    }
}


fn main() -> mlua::Result<()> {
    let definitions = Definitions::generate()
        .define("init", Definition::generate()
            .register::<SystemColor>("System")?
            .register::<Color>("Color")?
            .register::<Example>("Example")
            .value_with::<Example, _>("example", Some("Example module"))
            .function::<Color, ()>("printColor", ())
            .function_with::<String, (), _>("greet", (), |func| {
                func.document("Greet the name that was passed in");
                func.param(0, |param| param.name("name").doc("Name of the person to greet"));
            })
        )
        .finish();

    let gen = DefinitionFileGenerator::new(definitions);
    for (name, writer) in gen.iter() {
        // Writes to a new file `init.d.lua`
        writer.write_file(name).unwrap();
    }
    println!();
    Ok(())
}
```

Produces the following definition file

```lua
--- init.d.lua
--- @meta

--- @alias System "Black"
---  | "Red"
---  | "Green"
---  | "Yellow"
---  | "Blue"
---  | "Cyan"
---  | "Magenta"
---  | "White"

--- @alias Color SystemColor
---  | integer
---  | [integer, integer, integer]

--- This is a doc comment section for the overall type
--- @class Example
--- Example complex type
--- @field color Color

--- Example module
--- @type Example
example = nil

--- Greet the name that was passed in
--- @param name string Name of the person to greet
function greet(name) end

--- @param param0 Color
function printColor(param0) end
```
