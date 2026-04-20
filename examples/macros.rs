use mlua::{IntoLua, FromLua, Lua, StdLib};
use mlua_extras::{UserData, user_data_impl};

#[derive(Clone, UserData)]
struct Data {
    #[field(rename = 1)]
    name: String
}

#[user_data_impl]
impl Data {
    #[method]
    fn get_data(&self) -> mlua::Result<String> {
        Ok(self.name.clone())
    }

    /// This method is called first, if it returns a nil/none value then
    /// the auto implementation will fallback to it's implementation.
    /// 
    /// # Example
    /// 
    /// If this function returned a value for `1` then the auto impl with
    /// return that value. If this function returns nil then the auto impl
    /// will return the value for `self.name` since it is exposed at index `1`
    #[metamethod(Index)]
    fn index(&self, lua: &Lua, idx: isize) -> mlua::Result<mlua::Value> {
        match idx {
            -1 => "TESTING".into_lua(lua),
            _ => Ok(mlua::Value::Nil)
        }
    }


    /// This method is called first, if it returns an error value then
    /// the auto implementation will fallback to it's implementation.
    /// 
    /// # Example
    /// 
    /// If this function returned `Ok(())` for an index of `1` then the auto impl
    /// would just return that to the runtime. However, if this function returns
    /// an error value then the auto impl will attempt to use it's impl. If the
    /// auto impl doesn't now the index then the original error is returned.
    #[metamethod(NewIndex)]
    fn new_index(&mut self, lua: &Lua, idx: isize, value: mlua::Value) -> mlua::Result<()> {
        match idx {
            1 => self.name = <String as FromLua>::from_lua(value, lua)?,
            // It is recommended to return some sort of error from this implementation.
            //
            // This enforces strict indexing into userdata types and tells the auto impl
            // to fallback to it's implementation. If the internal implementation also fails
            // then the original error from this impl is returned to the lua runtime.
            _ => return Err(mlua::Error::runtime(format!("invalid index '{idx}'")))
        }
        Ok(())
    }
}

fn main() -> mlua::Result<()> {
    let lua = unsafe { Lua::unsafe_new_with(StdLib::ALL, Default::default()) };

    lua.globals().set("data", Data { name: "MluaExtras".into() })?;

    lua.load("
    print('Index [1]:', data[1])
    data[1] = 'HelloWorld'
    print('Set data[1] to \\'HelloWorld\\'')
    print('Get Data:', data:get_data())
    print('Index [-1]:', data[-1])
    ").exec()?;

    Ok(())
}
