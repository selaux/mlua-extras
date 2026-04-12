use std::path::PathBuf;
use mlua_extras::{
    mlua,
    typed::{
        generator::{Definition, DefinitionFileGenerator, Definitions},
        TypedDataFields, TypedDataMethods, TypedUserData,
    },
    Typed, UserData,
};

#[derive(Typed, UserData)]
struct A {}
impl TypedUserData for A {}

#[derive(Typed, UserData)]
struct B {}
impl TypedUserData for B {}

#[derive(Typed, UserData)]
#[typed(derive(A, B))]
enum Kind {
    A(String),
    B { name: String, data: String },
}

impl Kind {
    fn get_data(&self) -> String {
        match self {
            Self::A(data) => data.clone(),
            Self::B { data, .. } => data.clone(),
        }
    }
}

impl TypedUserData for Kind {
    fn add_fields<T: TypedDataFields<Self>>(fields: &mut T) {
        fields.add_field_method_get("name", |_lua, this: &Self| match this {
            Self::B { name, .. } => Ok(name.clone()),
            _ => Err(mlua::Error::runtime(
                "Kind does not contain field 'name' in it's current variant",
            )),
        });
        fields.add_field_method_get("data", |_lua, this: &Self| {
            match this {
                // Multiple variants with a field of the same name would convert the value into a lua value first and return it
                // This means that in the types it would be something like `data string|number` if there were multiple matches of different types
                Self::B { data, .. } => Ok(data.clone()),
                _ => Err(mlua::Error::runtime(
                    "Kind does not contain field 'data' in it's current variant",
                )),
            }
        });
    }

    fn add_methods<T: TypedDataMethods<Self>>(methods: &mut T) {
        methods.add_method("__variant", |_lua, this: &Self, _: ()| {
            Ok(match this {
                Self::A(_) => "A",
                Self::B { .. } => "B",
            })
        });

        methods.add_index_meta_method::<(String,), _, _, _>(|_lua, this: &Self, key: usize| {
            match key {
                1 => match this {
                    // Multiple variants with a tuple field of the same index would convert the value into a lua value first and return it
                    // This means that in the types it would be something like `[1] string|number` if there were multiple matches of different types
                    Self::A(value) => Ok(value.clone()),
                    _ => Err(mlua::Error::runtime(format!(
                        "Kind does not contain index '{key}' in it's current variant"
                    ))),
                },
                _ => Err(mlua::Error::runtime(format!(
                    "Kind does not contain index '{key}' in it's current variant"
                ))),
            }
        });

        methods.add_method("getData", |_lua, this: &Self, _: ()| Ok(this.get_data()))
    }
}

fn main() -> mlua::Result<()> {
    let lua = mlua::Lua::new();

    lua.globals().set("KindA", Kind::A("Test".into()))?;
    lua.globals().set(
        "KindB",
        Kind::B {
            name: "Test".into(),
            data: "Test Data".into(),
        },
    )?;

    if let Err(err) = lua
        .load(
            r#"
        print(KindA[1])
        print(KindA["1"])
        print(KindB.name)
        print(KindB.data)

        local ok, value = pcall(function () return KindA.name end)
        print('KindA.name', ok, tostring(value):match("(.-)\n") or tostring(value))

        ok, value = pcall(function () return KindA.data end)
        print('KindA.data', ok, tostring(value):match("(.-)\n") or tostring(value))
        
        ok, value = pcall(function() return KindB[1] end)
        print('KindB[1]', ok, tostring(value):match("(.-)\n") or tostring(value))

        print('KindA:getData()', KindA:getData())
        print('KindB:getData()', KindB:getData())
    "#,
        )
        .eval::<()>()
    {
        eprintln!("{err}");
    }

    let definitions: Definitions = Definitions::start()
        .define("tuple", Definition::start().register::<Kind>("Kind"))
        .finish();

    let types_path = PathBuf::from("examples/types");
    if !types_path.exists() {
        std::fs::create_dir_all(&types_path).unwrap();
    }

    let gen = DefinitionFileGenerator::new(definitions.clone());
    for (name, writer) in gen.iter() {
        println!("==== Generated \x1b[1;33mexample/types/{name}\x1b[0m ====");
        writer.write_file(types_path.join(name)).unwrap();
    }

    Ok(())
}
