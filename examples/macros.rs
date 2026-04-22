use std::path::PathBuf;

use mlua::{IntoLua, FromLua, Lua, StdLib};
use mlua_extras::{TypedUserData, UserData, typed::generator::{Definition, DefinitionFileGenerator, Definitions}, typed_user_data_impl, user_data_impl};

/// Structured Data
#[derive(Clone, TypedUserData)]
struct Data {
    /// Name of the data source
    name: String
}

#[typed_user_data_impl]
impl Data {
    #[method]
    fn get_data(&self) -> mlua::Result<String> {
        Ok(self.name.clone())
    }

    /// This method is called last.
    /// 
    /// use `#[field(skip)]` for fields that are assigned to the index
    /// to allow for them to overridden in this impl
    #[metamethod(Index)]
    fn index(&self, lua: &Lua, idx: isize) -> mlua::Result<mlua::Value> {
        match idx {
            -1 => "TESTING".into_lua(lua),
            1 => self.name.clone().into_lua(lua),
            _ => Ok(mlua::Value::Nil)
        }
    }
    
    /// This method is called last.
    /// 
    /// use `#[field(skip)]` for fields that are assigned to the index
    /// to allow for them to overridden in this impl
    #[metamethod(NewIndex)]
    fn new_index(&mut self, lua: &Lua, idx: isize, value: mlua::Value) -> mlua::Result<()> {
        match idx {
            1 => self.name = <String as FromLua>::from_lua(value, lua)?,
            // It is recommended to return some sort of error from this implementation.
            //
            // This enforces strict indexing into userdata types.
            _ => return Err(mlua::Error::runtime(format!("invalid index '{idx}'")))
        }
        Ok(())
    }
}

/// Kind of action
#[derive(Clone, TypedUserData)]
enum Custom {
    A,
    B(
        /// Variant B Data
        String
    ),
    C {
        name: String,
        /// Age of variant C
        age: u8,
    },
    D(
        /// Variant D Data
        u32
    ),
}

#[typed_user_data_impl]
impl Custom {
    /// Static field provided to Lua
    const COUNT: usize = 10;

    /// Get the direction [Getter]
    #[getter("direction")]
    fn get_direction(&self) -> String {
        "west".into()
    }

    /// Get the direction [Setter]
    #[setter("direction")]
    fn set_direction(&self, input: String) {
        _ = input;
    }

    /// Get the message based on the variant
    #[method]
    fn message(&self, input: String) -> String {
        match self {
            Self::A => "Hello, world!".into(),
            Self::B(msg) => msg.clone(),
            Self::C{ name, age } => format!("{name} age {age}"),
            Self::D(count) => count.to_string()
        }
    }
}

fn main() -> mlua::Result<()> {
    let lua = Lua::new();

    lua.globals().set("data", Data { name: "MluaExtras".into() })?;
    lua.globals().set("kind", Custom::A)?;

    lua.load("
    print('Index [1]:', data[1])
    data[1] = 'HelloWorld'
    print('Set data[1] to \\'HelloWorld\\'')
    print('Get Data:', data:get_data())
    print('Index [-1]:', data[-1])
    print('Kind:', kind._variant, kind:message())

    local ok, value = pcall(function () return kind[1] end)
    print('Kind [1]: OK', ok, tostring(value):match('(.-)\\n') or tostring(value))
    ").exec()?;

    let definitions: Definitions = Definitions::start()
        .define(
            "macros",
            Definition::start()
                .register::<Data>("Data")
                .register::<Custom>("Kind")
        )
        .finish();

    let types_path = PathBuf::from("examples/types");
    if !types_path.exists() {
        std::fs::create_dir_all(&types_path).unwrap();
    }

    let dfg = DefinitionFileGenerator::new(definitions.clone());
    for (name, writer) in dfg.iter() {
        println!("==== Generated \x1b[1;33mexample/types/{name}\x1b[0m ====");
        writer.write_file(types_path.join(name)).unwrap();
    }

    Ok(())
}
