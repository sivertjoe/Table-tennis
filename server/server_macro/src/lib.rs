use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};


const SUPPORTED_TYPES: [&'static str; 11] =
    ["isize", "i8,", "i16", "i32", "u8", "u16", "u32", "i64", "f64", "bool", "String"];


fn get_type(ty: &syn::Type) -> String
{
    match ty
    {
        syn::Type::Path(path) =>
        {
            let p = path.path.segments.first().unwrap();
            p.ident.to_string()
        },
        _ => panic!("UHH"),
    }
}

fn create_type(struct_name: String, names: Vec<String>, types: Vec<String>) -> String
{
    let mut builder = String::new();

    let _types = format!("let names: Vec<&str> = vec!{:?};\n", names);
    let mstn = "let map_single_to_name = |field: &str, cols: &Vec<&str>| -> Option<usize> { \
                cols.iter().position(|c| c == &field) };";

    let mctn = "let map_col_to_name = |cols: &Vec<&str>, fields: &Vec<&str>| -> \
                Vec<Option<usize>> { fields.iter().map(|f| map_single_to_name(f, cols)).collect() \
                };";

    builder.push_str(&_types);
    builder.push_str(mstn);
    builder.push_str(mctn);
    builder.push_str("let map = map_col_to_name(&row.column_names(), &names);\n");
    builder.push_str(&format!("Ok({} {{\n", &struct_name));

    let s: String = names
        .iter()
        .zip(types.iter())
        .enumerate()
        .map(|(i, (n, t))| {
            let _c = if SUPPORTED_TYPES.contains(&t.as_str())
            {
                format!("row.get(map[{}].expect(&format!(\"Error while trying to get field {{}} in Sql trait\", \"'{}'\")))?", i, n)
            }
            else
            {
                "Default::default()".to_string()
            };
            format!("{}: {},\n", n, _c)
        })
        .collect::<Vec<String>>()
        .join("");

    builder.push_str(&s);
    builder.push_str("})");

    builder
}

#[proc_macro_derive(Sql)]
pub fn implement_sql(item: TokenStream) -> TokenStream
{
    let input: DeriveInput = parse_macro_input!(item as DeriveInput);

    let fields = match &input.data
    {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields), ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };

    let struct_name = &input.ident;
    let field_name = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_string())
        .collect::<Vec<String>>();
    let field_type = fields.iter().map(|field| get_type(&field.ty)).collect::<Vec<String>>();

    let expr: proc_macro2::TokenStream =
        create_type(struct_name.to_string(), field_name, field_type).parse().unwrap();

    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics FromSql for #struct_name #ty_generics #where_clause
        {
            fn from_sql(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self>
            {
                #expr
            }
        }
    };

    expanded.into()
}
