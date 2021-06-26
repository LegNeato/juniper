use fnv::FnvHashMap;
use juniper::{
    execute, marker, DefaultScalarValue, EmptyMutation, EmptySubscription, FromInputValue,
    GraphQLInputObject, GraphQLType, GraphQLValue, InputValue, Registry, RootNode, ToInputValue,
    Value, Variables,
};

struct Root;
#[juniper::graphql_object]
impl Root {
    fn x(_input: [i32; 3]) -> bool {
        true
    }
}

#[derive(GraphQLInputObject, Debug, PartialEq)]
#[graphql(
    name = "MyInput",
    description = "input descr",
    scalar = DefaultScalarValue
)]
struct Input {
    regular_field: String,
    #[graphql(name = "haha", default = "33", description = "haha descr")]
    c: i32,

    #[graphql(default)]
    other: Option<bool>,
}

#[derive(GraphQLInputObject, Debug, PartialEq)]
#[graphql(rename = "none")]
struct NoRenameInput {
    regular_field: String,
}

#[derive(GraphQLInputObject, Debug, PartialEq)]
struct ArrayInput {
    data: [i32; 3],
}

/// Object comment.
#[derive(GraphQLInputObject, Debug, PartialEq)]
struct DocComment {
    /// Field comment.
    regular_field: bool,
}

/// Doc 1.\
/// Doc 2.
///
/// Doc 4.
#[derive(GraphQLInputObject, Debug, PartialEq)]
struct MultiDocComment {
    /// Field 1.
    /// Field 2.
    regular_field: bool,
}

/// This is not used as the description.
#[derive(GraphQLInputObject, Debug, PartialEq)]
#[graphql(description = "obj override")]
struct OverrideDocComment {
    /// This is not used as the description.
    #[graphql(description = "field override")]
    regular_field: bool,
}

#[derive(Debug, PartialEq)]
struct Fake;

impl<'a> marker::IsInputType<DefaultScalarValue> for &'a Fake {}

impl<'a> FromInputValue for &'a Fake {
    fn from_input_value(_v: &InputValue) -> Option<&'a Fake> {
        None
    }
}

impl<'a> ToInputValue for &'a Fake {
    fn to_input_value(&self) -> InputValue {
        InputValue::scalar("this is fake")
    }
}

impl<'a> GraphQLType<DefaultScalarValue> for &'a Fake {
    fn name(_: &()) -> Option<&'static str> {
        None
    }
    fn meta<'r>(_: &(), registry: &mut Registry<'r>) -> juniper::meta::MetaType<'r>
    where
        DefaultScalarValue: 'r,
    {
        let meta = registry.build_enum_type::<&'a Fake>(
            &(),
            &[juniper::meta::EnumValue {
                name: "fake".to_string(),
                description: None,
                deprecation_status: juniper::meta::DeprecationStatus::Current,
            }],
        );
        meta.into_meta()
    }
}

impl<'a> GraphQLValue<DefaultScalarValue> for &'a Fake {
    type Context = ();
    type TypeInfo = ();

    fn type_name<'i>(&self, info: &'i Self::TypeInfo) -> Option<&'i str> {
        <Self as GraphQLType>::name(info)
    }
}

#[derive(GraphQLInputObject, Debug, PartialEq)]
#[graphql(scalar = DefaultScalarValue)]
struct WithLifetime<'a> {
    regular_field: &'a Fake,
}

#[test]
fn validate_meta_info() {
    assert_eq!(
        <Input as GraphQLType<DefaultScalarValue>>::name(&()),
        Some("MyInput")
    );

    let mut registry: Registry = Registry::new(FnvHashMap::default());
    let meta = Input::meta(&(), &mut registry);
    assert_eq!(meta.name(), Some("MyInput"));
    assert_eq!(meta.description(), Some(&"input descr".to_string()));
}

#[test]
fn test_default_value_injection() {
    let input_no_defaults: InputValue = ::serde_json::from_value(serde_json::json!({
        "regularField": "a",
    }))
    .unwrap();

    let output_no_defaults: Input = FromInputValue::from_input_value(&input_no_defaults).unwrap();
    assert_eq!(
        output_no_defaults,
        Input {
            regular_field: "a".into(),
            c: 33,
            other: None,
        }
    );
}

#[test]
fn test_with_all_values_supplied() {
    let input: InputValue = ::serde_json::from_value(serde_json::json!({
        "regularField": "a",
        "haha": 55,
        "other": true,
    }))
    .unwrap();

    let output: Input = FromInputValue::from_input_value(&input).unwrap();
    assert_eq!(
        output,
        Input {
            regular_field: "a".into(),
            c: 55,
            other: Some(true),
        }
    );
}

#[test]
fn test_renaming_disabled() {
    let input: InputValue = ::serde_json::from_value(serde_json::json!({
        "regular_field": "hello",
    }))
    .unwrap();

    let output: NoRenameInput = FromInputValue::from_input_value(&input).unwrap();
    assert_eq!(
        output,
        NoRenameInput {
            regular_field: "hello".into(),
        }
    );
}

#[test]
fn test_array() {
    let input: InputValue = ::serde_json::from_value(serde_json::json!({
        "data": [1, 2, 3],
    }))
    .unwrap();

    let output: ArrayInput = FromInputValue::from_input_value(&input).unwrap();
    assert_eq!(output, ArrayInput { data: [1, 2, 3] });
}

#[tokio::test]
async fn test_array_query() {
    let doc = r#"
        {
            x(input: [1, 2, 3])
        }"#;

    let schema = RootNode::new(
        Root,
        EmptyMutation::<()>::new(),
        EmptySubscription::<()>::new(),
    );

    assert_eq!(
        execute(doc, None, &schema, &Variables::new(), &()).await,
        Ok((
            Value::object(vec![("x", Value::scalar(true))].into_iter().collect()),
            vec![]
        ))
    );
}

#[tokio::test]
async fn test_array_type_mismatch() {
    let doc = r#"
        {
            x(input: ["foo", "bar", "baz"])
        }"#;

    let schema = RootNode::new(
        Root,
        EmptyMutation::<()>::new(),
        EmptySubscription::<()>::new(),
    );

    assert!(execute(doc, None, &schema, &Variables::new(), &())
        .await
        .is_err());
}

#[tokio::test]
async fn test_array_length_mismatch() {
    let doc = r#"
        {
            x(input: [1, 2])
        }"#;

    let schema = RootNode::new(
        Root,
        EmptyMutation::<()>::new(),
        EmptySubscription::<()>::new(),
    );

    assert!(execute(doc, None, &schema, &Variables::new(), &())
        .await
        .is_err());
}

#[test]
fn test_doc_comment() {
    let mut registry: Registry = Registry::new(FnvHashMap::default());
    let meta = DocComment::meta(&(), &mut registry);
    assert_eq!(meta.description(), Some(&"Object comment.".to_string()));
}

#[test]
fn test_multi_doc_comment() {
    let mut registry: Registry = Registry::new(FnvHashMap::default());
    let meta = MultiDocComment::meta(&(), &mut registry);
    assert_eq!(
        meta.description(),
        Some(&"Doc 1. Doc 2.\n\nDoc 4.".to_string())
    );
}

#[test]
fn test_doc_comment_override() {
    let mut registry: Registry = Registry::new(FnvHashMap::default());
    let meta = OverrideDocComment::meta(&(), &mut registry);
    assert_eq!(meta.description(), Some(&"obj override".to_string()));
}
