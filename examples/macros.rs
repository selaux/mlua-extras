use mlua::{IntoLua, FromLua, Lua, StdLib};
use mlua_extras::{UserData, Typed, user_data_impl};

#[derive(Clone, UserData)]
struct Data {
    name: String
}

#[user_data_impl]
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

#[derive(Clone, UserData)]
enum Kind {
    A,
    B(String),
    C {
        name: String,
        age: u8,
    },
    D(u32),
}

#[user_data_impl]
impl Kind {
    #[method]
    fn message(&self) -> String {
        match self {
            Self::A => "Hello, world!".into(),
            Self::B(msg) => msg.clone(),
            Self::C{ name, age } => format!("{name} age {age}"),
            Self::D(count) => count.to_string()
        }
    }
}

fn main() -> mlua::Result<()> {
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, Default::default()) };

    lua.globals().set("data", Data { name: "MluaExtras".into() })?;
    lua.globals().set("kind", Kind::A)?;

    lua.load("
    print('Index [1]:', data[1])
    data[1] = 'HelloWorld'
    print('Set data[1] to \\'HelloWorld\\'')
    print('Get Data:', data:get_data())
    print('Index [-1]:', data[-1])
    print('Kind:', kind._variant, kind:message())

    local ok, value = pcall(function() return kind[1] end)
    print('Kind [1]: OK', ok, value)
    ").exec()?;

    Ok(())
}
