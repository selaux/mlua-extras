use mlua::{
    AnyUserData, FromLua, FromLuaMulti, IntoLua, IntoLuaMulti, Lua, UserData,
    UserDataFields, UserDataMethods,
};

use crate::MaybeSend;

use super::{Typed, TypedDataFields, TypedDataMethods, TypedMultiValue};

/// Wrapper around a [`UserDataFields`] and [`UserDataMethods`]
/// to allow [`TypedUserData`] implementations to be used for [`UserData`]
/// implementations
pub struct WrappedBuilder<'ctx, U>(&'ctx mut U);
impl<'ctx, U> WrappedBuilder<'ctx, U> {
    pub fn new(u: &'ctx mut U) -> Self {
        WrappedBuilder(u)
    }
}

impl<'ctx, T: UserData, U: UserDataFields<T>> TypedDataFields<T> for WrappedBuilder<'ctx, U> {
    fn document(&mut self, _doc: &str) -> &mut Self {
        self
    }

    fn add_field<V>(&mut self, name: impl Into<String>, value: V)
    where
        V: IntoLua + Clone + 'static + Typed,
    {
        self.0.add_field(name, value)
    }

    fn add_field_function_set<S, A, F>(&mut self, name: S, function: F)
    where
        S: Into<String>,
        A: FromLua + Typed,
        F: FnMut(&Lua, AnyUserData, A) -> mlua::Result<()> + MaybeSend + 'static,
    {
        self.0.add_field_function_set(name, function)
    }

    fn add_field_function_get<S, R, F>(&mut self, name: S, function: F)
    where
        S: Into<String>,
        R: IntoLua + Typed,
        F: Fn(&Lua, AnyUserData) -> mlua::Result<R> + MaybeSend + 'static,
    {
        self.0.add_field_function_get(name, function)
    }

    fn add_field_function_get_set<S, R, A, GET, SET>(&mut self, name: S, get: GET, set: SET)
    where
        S: Into<String>,
        R: IntoLua + Typed,
        A: FromLua + Typed,
        GET: Fn(&Lua, AnyUserData) -> mlua::Result<R> + MaybeSend + 'static,
        SET: Fn(&Lua, AnyUserData, A) -> mlua::Result<()> + MaybeSend + 'static,
    {
        let name: String = name.into();
        self.0.add_field_function_get(&name, get);
        self.0.add_field_function_set(name, set);
    }

    fn add_field_method_set<S, A, M>(&mut self, name: S, method: M)
    where
        S: Into<String>,
        A: FromLua + Typed,
        M: FnMut(&Lua, &mut T, A) -> mlua::Result<()> + MaybeSend + 'static,
    {
        self.0.add_field_method_set(name, method)
    }

    fn add_field_method_get<S, R, M>(&mut self, name: S, method: M)
    where
        S: Into<String>,
        R: IntoLua + Typed,
        M: Fn(&Lua, &T) -> mlua::Result<R> + MaybeSend + 'static,
    {
        self.0.add_field_method_get(name, method)
    }

    fn add_field_method_get_set<S, R, A, GET, SET>(&mut self, name: S, get: GET, set: SET)
    where
        S: Into<String>,
        R: IntoLua + Typed,
        A: FromLua + Typed,
        GET: Fn(&Lua, &T) -> mlua::Result<R> + MaybeSend + 'static,
        SET: Fn(&Lua, &mut T, A) -> mlua::Result<()> + MaybeSend + 'static,
    {
        let name: String = name.into();
        self.0.add_field_method_get(&name, get);
        self.0.add_field_method_set(name, set);
    }

    fn add_meta_field<V>(&mut self, meta: impl Into<String>, value: V)
    where
        V: IntoLua + 'static,
    {
        self.0.add_meta_field(meta, value)
    }

    fn add_meta_field_with<R, F>(&mut self, name: impl Into<String>, f: F)
        where
            F: 'static + MaybeSend + Fn(&Lua) -> mlua::Result<R>,
            R: IntoLua + 'static {
        self.0.add_meta_field_with(name, f);
    }
}

impl<'ctx, T: UserData, U: UserDataMethods<T>> TypedDataMethods<T> for WrappedBuilder<'ctx, U> {
    fn document(&mut self, _documentation: &str) -> &mut Self {
        self
    }

    fn param<S: std::fmt::Display, D: std::fmt::Display>(
        &mut self,
        _name: S,
        _doc: D,
    ) -> &mut Self {
        self
    }

    fn ret<S: std::fmt::Display>(&mut self, _doc: S) -> &mut Self {
        self
    }

    fn add_method<S, A, R, M>(&mut self, name: S, method: M)
    where
        S: Into<String>,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        M: Fn(&Lua, &T, A) -> mlua::Result<R> + MaybeSend + 'static,
    {
        self.0.add_method(name, method)
    }

    fn add_function<S, A, R, F>(&mut self, name: S, function: F)
    where
        S: Into<String>,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        F: Fn(&Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
    {
        self.0.add_function(name, function)
    }

    fn add_method_mut<S, A, R, M>(&mut self, name: S, method: M)
    where
        S: Into<String>,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        M: FnMut(&Lua, &mut T, A) -> mlua::Result<R> + MaybeSend + 'static,
    {
        self.0.add_method_mut(name, method)
    }

    fn add_meta_method<A, R, M>(&mut self, meta: impl Into<String>, method: M)
    where
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        M: 'static + MaybeSend + Fn(&Lua, &T, A) -> mlua::Result<R>,
    {
        self.0.add_meta_method(meta, method)
    }

    #[cfg(feature = "async")]
    fn add_async_method<'s, S: ?Sized + AsRef<str>, A, R, M, MR>(&mut self, name: &S, method: M)
    where
        T: 'static,
        M: Fn(&Lua, &'s T, A) -> MR + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        MR: std::future::Future<Output = mlua::Result<R>> + 's,
        R: IntoLuaMulti,
    {
        self.0.add_async_method(name, method)
    }

    #[cfg(feature = "async")]
    fn add_async_method_mut<'s, S: ?Sized + AsRef<str>, A, R, M, MR>(&mut self, name: &S, method: M)
    where
        T: 'static,
        M: Fn(&Lua, &'s mut T, A) -> MR + MaybeSend + 'static,
        A: FromLuaMulti + TypedMultiValue,
        MR: std::future::Future<Output = mlua::Result<R>> + 's,
        R: IntoLuaMulti,
    {
        self.0.add_async_method_mut(name, method)
    }

    fn add_function_mut<S, A, R, F>(&mut self, name: S, function: F)
    where
        S: Into<String>,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        F: FnMut(&Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
    {
        self.0.add_function_mut(name, function)
    }

    fn add_meta_function<A, R, F>(&mut self, meta: impl Into<String>, function: F)
    where
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        F: Fn(&Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
    {
        self.0.add_meta_function(meta, function)
    }

    #[cfg(feature = "async")]
    fn add_async_function<S: ?Sized, A, R, F, FR>(&mut self, name: &S, function: F)
    where
        S: AsRef<str>,
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        F: 'static + MaybeSend + Fn(&Lua, A) -> FR,
        FR: std::future::Future<Output = mlua::Result<R>>,
    {
        self.0.add_async_function(name, function)
    }

    fn add_meta_method_mut<A, R, M>(&mut self, meta: impl Into<String>, method: M)
    where
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        M: 'static + MaybeSend + FnMut(&Lua, &mut T, A) -> mlua::Result<R>,
    {
        self.0.add_meta_method_mut(meta, method)
    }

    fn add_meta_function_mut<A, R, F>(&mut self, meta: impl Into<String>, function: F)
    where
        A: FromLuaMulti + TypedMultiValue,
        R: IntoLuaMulti + TypedMultiValue,
        F: FnMut(&Lua, A) -> mlua::Result<R> + MaybeSend + 'static,
    {
        self.0.add_meta_function_mut(meta, function)
    }
}
