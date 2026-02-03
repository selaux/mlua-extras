use mlua_extras::{
    extras::{LuaExtras, Require},
    mlua::{self, Function, Lua, Table, UserData, Value, Variadic},
};

struct MyModule;
impl UserData for MyModule {
    fn add_fields<F: mlua::prelude::LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field("data", "Some Data");
    }

    fn add_methods<M: mlua::prelude::LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("print", |_lua, values: Variadic<Value>| {
            println!(
                "{}",
                values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<mlua::Result<Vec<_>>>()?
                    .join(" ")
            );
            Ok(())
        });
    }
}

const CODE: &str = r#"data = {
    first = "key",
    second = "value"
}

mymodule.print(mymodule.data)
"#;

fn main() -> mlua::Result<()> {
    let lua = Lua::new();

    // Get a value in a nested module/table (trait LuaExtras)
    let table = lua.require::<Table>("table")?;
    // Also works with regular tables (trait Require)
    let _unpack = table.require::<Function>("unpack")?;

    // Import a module into lua's global scope. This is just a UserData
    lua.set_global("mymodule", MyModule)?;

    {
        // Importing also works with tables given a lua context
        let temp = lua.create_table()?;
        temp.set("mymodule", MyModule)?;
    }

    if let Err(err) = lua.load(CODE).eval::<Value>() {
        eprintln!("{err}");
    }

    Ok(())
}
