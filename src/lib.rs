//! # Json path
//! The library provides the basic functionality
//! to find the slice of data according to the query.
//! The idea comes from xpath for xml structures.
//! The details can be found over [`there`]
//! Therefore JSONPath is a query language for JSON,
//! similar to XPath for XML. The jsonpath query is a set of assertions to specify the JSON fields that need to be verified.
//!
//! # Simple example
//! Let's suppose we have a following json:
//! ```json
//!  {
//!   "shop": {
//!    "orders": [
//!       {"id": 1, "active": true},
//!       {"id": 2 },
//!       {"id": 3 },
//!       {"id": 4, "active": true}
//!     ]
//!   }
//! }
//! ```
//! And we pursue to find all orders id having the field 'active'
//! we can construct the jsonpath instance like that
//! ```$.shop.orders[?(@.active)].id``` and get the result ``` [1,4] ```
//!
//! # Another examples
//! ```json
//! { "store": {
//!     "book": [
//!       { "category": "reference",
//!         "author": "Nigel Rees",
//!         "title": "Sayings of the Century",
//!         "price": 8.95
//!       },
//!       { "category": "fiction",
//!         "author": "Evelyn Waugh",
//!         "title": "Sword of Honour",
//!         "price": 12.99
//!       },
//!       { "category": "fiction",
//!         "author": "Herman Melville",
//!         "title": "Moby Dick",
//!         "isbn": "0-553-21311-3",
//!         "price": 8.99
//!       },
//!       { "category": "fiction",
//!         "author": "J. R. R. Tolkien",
//!         "title": "The Lord of the Rings",
//!         "isbn": "0-395-19395-8",
//!         "price": 22.99
//!       }
//!     ],
//!     "bicycle": {
//!       "color": "red",
//!       "price": 19.95
//!     }
//!   }
//! }
//! ```
//! and examples
//! - ``` $.store.book[*].author ``` : the authors of all books in the store
//! - ``` $..book[?(@.isbn)]``` : filter all books with isbn number
//! - ``` $..book[?(@.price<10)]``` : filter all books cheapier than 10
//! - ``` $..*``` : all Elements in XML document. All members of JSON structure
//! - ``` $..book[0,1]``` : The first two books
//! - ``` $..book[:2]``` : The first two books
//!
//! # Operators
//!
//! - `$` : Pointer to the root of the json. It is gently advising to start every jsonpath from the root. Also, inside the filters to point out that the path is starting from the root.
//! - `@`Pointer to the current element inside the filter operations.It is used inside the filter operations to iterate the collection.
//! - `*` or `[*]`Wildcard. It brings to the list all objects and elements regardless their names.It is analogue a flatmap operation.
//! - `<..>`| Descent operation. It brings to the list all objects, children of that objects and etc It is analogue a flatmap operation.
//! - `.<name>` or `.['<name>']`the key pointing to the field of the objectIt is used to obtain the specific field.
//! - `['<name>' (, '<name>')]`the list of keysthe same usage as for a single key but for list
//! - `[<number>]`the filter getting the element by its index.
//! - `[<number> (, <number>)]`the list if elements of array according to their indexes representing these numbers. |
//! - `[<start>:<end>:<step>]`slice operator to get a list of element operating with their indexes. By default step = 1, start = 0, end = array len. The elements can be omitted ```[:]```
//! - `[?(<expression>)]`the logical expression to filter elements in the list.It is used with arrays preliminary.
//!
//! # Examples
//!```rust
//! use std::str::FromStr;
//! use serde_json::{json, Value};
//! use jsonpath_rust::{jp_v, find_slice, JsonPathValue, JsonPathInst};
//!
//! fn test() -> Result<(), Box<dyn std::error::Error>> {
//!     let json = serde_json::from_str(r#"{"first":{"second":[{"active":1},{"passive":1}]}}"#)?;
//!     let path = JsonPathInst::from_str("$.first.second[?(@.active)]")?;
//!     let slice_of_data:Vec<JsonPathValue<Value>> = find_slice(&path, &json);
//!     let js = json!({"active":1});
//!     assert_eq!(slice_of_data, jp_v![&js;"$.first.second[0]",]);
//!     # Ok(())
//! }
//! ```
//!
//!
//! [`there`]: https://goessner.net/articles/JsonPath/

#![allow(clippy::vec_init_then_push)]

use crate::parser::model::JsonPath;
use crate::parser::parser::parse_json_path;
use crate::path::json_path_instance;
use parser::errors::JsonPathParserError;
use serde_json::Value;
use std::convert::TryInto;
use std::fmt::Debug;
use std::ops::Deref;
use std::str::FromStr;
use JsonPathValue::{NewValue, NoValue, Slice};

pub mod parser;
pub mod path;

#[macro_use]
extern crate pest_derive;
extern crate core;
extern crate pest;

/// the trait allows to mix the method path to the value of [Value]
/// and thus the using can be shortened to the following one:
/// # Examples:
/// ```
/// use std::str::FromStr;
/// use serde_json::{json, Value};
/// use jsonpath_rust::jp_v;
/// use jsonpath_rust::{find_slice, JsonPathQuery, JsonPathInst, JsonPathValue};
///
/// fn test() -> Result<(), Box<dyn std::error::Error>> {
///     let json: Value = serde_json::from_str("{}")?;
///     let v = json.path("$..book[?(@.author size 10)].title")?;
///     assert_eq!(v, json!([]));
///
///     let json: Value = serde_json::from_str("{}")?;
///     let path = json.path("$..book[?(@.author size 10)].title")?;
///
///     assert_eq!(path, json!(["Sayings of the Century"]));
///
///     let json: Value = serde_json::from_str("{}")?;
///     let path: JsonPathInst = JsonPathInst::from_str("$..book[?(@.author size 10)].title")?;
///
///     let v = find_slice(&path, &json);
///     let js = json!("Sayings of the Century");
///     assert_eq!(v, jp_v![&js;"",]);
///     # Ok(())
/// }
///
/// ```
/// #Note:
/// the result is going to be cloned and therefore it can be significant for the huge queries
pub trait JsonPathQuery {
    fn path(self, query: &str) -> Result<Value, JsonPathParserError>;
}

#[derive(Clone, Debug)]
pub struct JsonPathInst {
    inner: JsonPath,
}

impl FromStr for JsonPathInst {
    type Err = JsonPathParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(JsonPathInst {
            inner: s.try_into()?,
        })
    }
}

impl JsonPathInst {
    pub fn find_slice<'a>(&'a self, value: &'a Value) -> Vec<JsonPtr<'a, Value>> {
        use crate::path::Path;
        json_path_instance(&self.inner, value)
            .find(JsonPathValue::from_root(value))
            .into_iter()
            .filter(|v| v.has_value())
            .map(|v| match v {
                JsonPathValue::Slice(v, _) => JsonPtr::Slice(v),
                JsonPathValue::NewValue(v) => JsonPtr::NewValue(v),
                JsonPathValue::NoValue => unreachable!("has_value was already checked"),
            })
            .collect()
    }
}

/// Json paths may return either pointers to the original json or new data. This custom pointer type allows us to handle both cases.
/// Unlike JsonPathValue, this type does not represent NoValue to allow the implementation of Deref.
pub enum JsonPtr<'a, Data> {
    /// The slice of the initial json data
    Slice(&'a Data),
    /// The new data that was generated from the input data (like length operator)
    NewValue(Data),
}

/// Allow deref from json pointer to value.
impl<'a> Deref for JsonPtr<'a, Value> {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        match self {
            JsonPtr::Slice(v) => v,
            JsonPtr::NewValue(v) => v,
        }
    }
}

impl JsonPathQuery for Value {
    fn path(self, query: &str) -> Result<Value, JsonPathParserError> {
        let p = JsonPathInst::from_str(query)?;
        Ok(find(&p, &self))
    }
}

/*
impl<T> JsonPathQuery for T
    where T: Deref<Target=Value> {
    fn path(self, query: &str) -> Result<Value, String> {
        let p = JsonPathInst::from_str(query)?;
        Ok(find(&p, self.deref()))
    }
}
 */

/// just to create a json path value of data
/// Example:
///  - `jp_v(&json) = JsonPathValue::Slice(&json)`
///  - `jp_v(&json;"foo") = JsonPathValue::Slice(&json, "foo".to_string())`
///  - `jp_v(&json,) = vec![JsonPathValue::Slice(&json)]`
///  - `jp_v[&json1,&json1] = vec![JsonPathValue::Slice(&json1),JsonPathValue::Slice(&json2)]`
///  - `jp_v(json) = JsonPathValue::NewValue(json)`
/// ```
/// use std::str::FromStr;
/// use serde_json::{json, Value};
/// use jsonpath_rust::{jp_v, find_slice, JsonPathQuery, JsonPathInst, JsonPathValue};
///
/// fn test() -> Result<(), Box<dyn std::error::Error>> {
///     let json: Value = serde_json::from_str("{}")?;
///     let path: JsonPathInst = JsonPathInst::from_str("$..book[?(@.author size 10)].title")?;
///     let v = find_slice(&path, &json);
///
///     let js = json!("Sayings of the Century");
///     assert_eq!(v, jp_v![&js;"",]);
///     # Ok(())
/// }
/// ```
#[macro_export]
macro_rules! jp_v {
    (&$v:expr) =>{
        JsonPathValue::Slice(&$v, String::new())
    };

    (&$v:expr ; $s:expr) =>{
        JsonPathValue::Slice(&$v, $s.to_string())
    };

    ($(&$v:expr;$s:expr),+ $(,)?) =>{
        {
        let mut res = Vec::new();
        $(
           res.push(jp_v!(&$v ; $s));
        )+
        res
        }
    };

    ($(&$v:expr),+ $(,)?) => {
        {
        let mut res = Vec::new();
        $(
           res.push(jp_v!(&$v));
        )+
        res
        }
    };

    ($v:expr) =>{
        JsonPathValue::NewValue($v)
    };

}

/// Represents the path of the found json data
type JsPathStr = String;

pub(crate) fn jsp_idx(prefix: &str, idx: usize) -> String {
    format!("{}[{}]", prefix, idx)
}
pub(crate) fn jsp_obj(prefix: &str, key: &str) -> String {
    format!("{}.['{}']", prefix, key)
}

/// A result of json path
/// Can be either a slice of initial data or a new generated value(like length of array)
#[derive(Debug, PartialEq, Clone)]
pub enum JsonPathValue<'a, Data> {
    /// The slice of the initial json data
    Slice(&'a Data, JsPathStr),
    /// The new data that was generated from the input data (like length operator)
    NewValue(Data),
    /// The absent value that indicates the input data is not matched to the given json path (like the absent fields)
    NoValue,
}

impl<'a, Data: Clone + Debug + Default> JsonPathValue<'a, Data> {
    /// Transforms given value into data either by moving value out or by cloning
    pub fn to_data(self) -> Data {
        match self {
            Slice(r, _) => r.clone(),
            NewValue(val) => val,
            NoValue => Data::default(),
        }
    }

    /// Transforms given value into path
    pub fn to_path(self) -> Option<JsPathStr> {
        match self {
            Slice(_, path) => Some(path),
            _ => None,
        }
    }

    pub fn from_root(data: &'a Data) -> Self {
        Slice(data, String::from("$"))
    }
    pub fn new_slice(data: &'a Data, path: String) -> Self {
        Slice(data, path.to_string())
    }
}

impl<'a, Data> JsonPathValue<'a, Data> {
    fn only_no_value(input: &[JsonPathValue<'a, Data>]) -> bool {
        !input.is_empty() && input.iter().filter(|v| v.has_value()).count() == 0
    }

    fn map_vec(data: Vec<(&'a Data, JsPathStr)>) -> Vec<JsonPathValue<'a, Data>> {
        data.into_iter()
            .map(|(data, pref)| Slice(data, pref))
            .collect()
    }

    fn map_slice<F>(self, mapper: F) -> Vec<JsonPathValue<'a, Data>>
    where
        F: FnOnce(&'a Data, JsPathStr) -> Vec<(&'a Data, JsPathStr)>,
    {
        match self {
            Slice(r, pref) => mapper(r, pref)
                .into_iter()
                .map(|(d, s)| Slice(d, s))
                .collect(),

            NewValue(_) => vec![],
            no_v => vec![no_v],
        }
    }

    fn flat_map_slice<F>(self, mapper: F) -> Vec<JsonPathValue<'a, Data>>
    where
        F: FnOnce(&'a Data, JsPathStr) -> Vec<JsonPathValue<'a, Data>>,
    {
        match self {
            Slice(r, pref) => mapper(r, pref),
            _ => vec![NoValue],
        }
    }

    pub fn has_value(&self) -> bool {
        !matches!(self, NoValue)
    }

    pub fn vec_as_data(input: Vec<JsonPathValue<'a, Data>>) -> Vec<&'a Data> {
        input
            .into_iter()
            .filter_map(|v| match v {
                Slice(el, _) => Some(el),
                _ => None,
            })
            .collect()
    }
    pub fn vec_as_pair(input: Vec<JsonPathValue<'a, Data>>) -> Vec<(&'a Data, JsPathStr)> {
        input
            .into_iter()
            .filter_map(|v| match v {
                Slice(el, v) => Some((el, v)),
                _ => None,
            })
            .collect()
    }

    /// moves a pointer (from slice) out or provides a default value when the value was generated
    pub fn slice_or(self, default: &'a Data) -> &'a Data {
        match self {
            Slice(r, _) => r,
            NewValue(_) | NoValue => default,
        }
    }
}

/// finds a slice of data in the set json.
/// The result is a vector of references to the incoming structure.
///
/// In case, if there is no match `find_slice` will return `vec![NoValue]`.
///
/// ## Example
/// ```rust
/// use jsonpath_rust::{JsonPathInst, JsonPathValue};
/// use serde_json::json;
/// # use std::str::FromStr;
///
/// let data = json!({"first":{"second":[{"active":1},{"passive":1}]}});
/// let path = JsonPathInst::from_str("$.first.second[?(@.active)]").unwrap();
/// let slice_of_data = jsonpath_rust::find_slice(&path, &data);
///
/// let expected_value = json!({"active":1});
/// let expected_path = "$.['first'].['second'][0]".to_string();
///
/// assert_eq!(
///     slice_of_data,
///     vec![JsonPathValue::Slice(&expected_value, expected_path)]
/// );
/// ```
pub fn find_slice<'a>(path: &'a JsonPathInst, json: &'a Value) -> Vec<JsonPathValue<'a, Value>> {
    use crate::path::Path;
    let instance = json_path_instance(&path.inner, json);
    let res = instance.find(JsonPathValue::from_root(json));
    let has_v: Vec<JsonPathValue<'_, Value>> = res.into_iter().filter(|v| v.has_value()).collect();

    if has_v.is_empty() {
        vec![NoValue]
    } else {
        has_v
    }
}

/// finds a slice of data and wrap it with Value::Array by cloning the data.
/// Returns either an array of elements or Json::Null if the match is incorrect.
///
/// In case, if there is no match `find` will return `json!(null)`.
///
/// ## Example
/// ```rust
/// use jsonpath_rust::{JsonPathInst, JsonPathValue};
/// use serde_json::{Value, json};
/// # use std::str::FromStr;
///
/// let data = json!({"first":{"second":[{"active":1},{"passive":1}]}});
/// let path = JsonPathInst::from_str("$.first.second[?(@.active)]").unwrap();
/// let cloned_data = jsonpath_rust::find(&path, &data);
///
/// assert_eq!(cloned_data, Value::Array(vec![json!({"active":1})]));
/// ```
pub fn find(path: &JsonPathInst, json: &Value) -> Value {
    let slice = find_slice(path, json);
    if !slice.is_empty() {
        if JsonPathValue::only_no_value(&slice) {
            Value::Null
        } else {
            Value::Array(
                slice
                    .into_iter()
                    .filter(|v| v.has_value())
                    .map(|v| v.to_data())
                    .collect(),
            )
        }
    } else {
        Value::Array(vec![])
    }
}

/// finds a path describing the value, instead of the value itself.
/// If the values has been obtained by moving the data out of the initial json the path is absent.
///
/// ** If the value has been modified during the search, there is no way to find a path of a new value.
/// It can happen if we try to find a length() of array, for in stance.**
///
/// ## Example
/// ```rust
/// use jsonpath_rust::{JsonPathInst, JsonPathValue};
/// use serde_json::{Value, json};
/// # use std::str::FromStr;
///
/// let data = json!({"first":{"second":[{"active":1},{"passive":1}]}});
/// let path = JsonPathInst::from_str("$.first.second[?(@.active)]").unwrap();
/// let slice_of_data: Value = jsonpath_rust::find_as_path(&path, &data);
///
/// let expected_path = "$.['first'].['second'][0]".to_string();
/// assert_eq!(slice_of_data, Value::Array(vec![Value::String(expected_path)]));
/// ```
pub fn find_as_path(path: &JsonPathInst, json: &Value) -> Value {
    Value::Array(
        find_slice(path, json)
            .into_iter()
            .flat_map(|v| v.to_path())
            .map(|v| v.into())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use crate::JsonPathQuery;
    use crate::JsonPathValue::{NoValue, Slice};
    use crate::{jp_v, JsonPathInst, JsonPathValue};
    use serde_json::{json, Value};
    use std::ops::Deref;
    use std::str::FromStr;

    fn test(json: &str, path: &str, expected: Vec<JsonPathValue<Value>>) {
        let json: Value = match serde_json::from_str(json) {
            Ok(json) => json,
            Err(e) => panic!("error while parsing json: {}", e),
        };
        let path = match JsonPathInst::from_str(path) {
            Ok(path) => path,
            Err(e) => panic!("error while parsing jsonpath: {}", e),
        };

        assert_eq!(super::find_slice(&path, &json), expected)
    }

    fn template_json<'a>() -> &'a str {
        r#" {"store": { "book": [
             {
                 "category": "reference",
                 "author": "Nigel Rees",
                 "title": "Sayings of the Century",
                 "price": 8.95
             },
             {
                 "category": "fiction",
                 "author": "Evelyn Waugh",
                 "title": "Sword of Honour",
                 "price": 12.99
             },
             {
                 "category": "fiction",
                 "author": "Herman Melville",
                 "title": "Moby Dick",
                 "isbn": "0-553-21311-3",
                 "price": 8.99
             },
             {
                 "category": "fiction",
                 "author": "J. R. R. Tolkien",
                 "title": "The Lord of the Rings",
                 "isbn": "0-395-19395-8",
                 "price": 22.99
             }
         ],
         "bicycle": {
             "color": "red",
             "price": 19.95
         }
     },
     "array":[0,1,2,3,4,5,6,7,8,9],
     "orders":[
         {
             "ref":[1,2,3],
             "id":1,
             "filled": true
         },
         {
             "ref":[4,5,6],
             "id":2,
             "filled": false
         },
         {
             "ref":[7,8,9],
             "id":3,
             "filled": null
         }
      ],
     "expensive": 10 }"#
    }

    #[test]
    fn simple_test() {
        let j1 = json!(2);
        test("[1,2,3]", "$[1]", jp_v![&j1;"$[1]",]);
    }

    #[test]
    fn root_test() {
        let js = serde_json::from_str(template_json()).unwrap();
        test(template_json(), "$", jp_v![&js;"$",]);
    }

    #[test]
    fn descent_test() {
        let v1 = json!("reference");
        let v2 = json!("fiction");
        test(
            template_json(),
            "$..category",
            jp_v![
                 &v1;"$.['store'].['book'][0].['category']",
                 &v2;"$.['store'].['book'][1].['category']",
                 &v2;"$.['store'].['book'][2].['category']",
                 &v2;"$.['store'].['book'][3].['category']",],
        );
        let js1 = json!(19.95);
        let js2 = json!(8.95);
        let js3 = json!(12.99);
        let js4 = json!(8.99);
        let js5 = json!(22.99);
        test(
            template_json(),
            "$.store..price",
            jp_v![
                &js1;"$.['store'].['bicycle'].['price']",
                &js2;"$.['store'].['book'][0].['price']",
                &js3;"$.['store'].['book'][1].['price']",
                &js4;"$.['store'].['book'][2].['price']",
                &js5;"$.['store'].['book'][3].['price']",
            ],
        );
        let js1 = json!("Nigel Rees");
        let js2 = json!("Evelyn Waugh");
        let js3 = json!("Herman Melville");
        let js4 = json!("J. R. R. Tolkien");
        test(
            template_json(),
            "$..author",
            jp_v![
            &js1;"$.['store'].['book'][0].['author']",
            &js2;"$.['store'].['book'][1].['author']",
            &js3;"$.['store'].['book'][2].['author']",
            &js4;"$.['store'].['book'][3].['author']",],
        );
    }

    #[test]
    fn wildcard_test() {
        let js1 = json!("reference");
        let js2 = json!("fiction");
        test(
            template_json(),
            "$..book.[*].category",
            jp_v![
                &js1;"$.['store'].['book'][0].['category']",
                &js2;"$.['store'].['book'][1].['category']",
                &js2;"$.['store'].['book'][2].['category']",
                &js2;"$.['store'].['book'][3].['category']",],
        );
        let js1 = json!("Nigel Rees");
        let js2 = json!("Evelyn Waugh");
        let js3 = json!("Herman Melville");
        let js4 = json!("J. R. R. Tolkien");
        test(
            template_json(),
            "$.store.book[*].author",
            jp_v![
                &js1;"$.['store'].['book'][0].['author']",
                &js2;"$.['store'].['book'][1].['author']",
                &js3;"$.['store'].['book'][2].['author']",
                &js4;"$.['store'].['book'][3].['author']",],
        );
    }

    #[test]
    fn descendent_wildcard_test() {
        let js1 = json!("Moby Dick");
        let js2 = json!("The Lord of the Rings");
        test(
            template_json(),
            "$..*.[?(@.isbn)].title",
            jp_v![
                &js1;"$.['store'].['book'][2].['title']",
                &js2;"$.['store'].['book'][3].['title']",
                &js1;"$.['store'].['book'][2].['title']",
                &js2;"$.['store'].['book'][3].['title']"],
        );
    }

    #[test]
    fn field_test() {
        let value = json!({"active":1});
        test(
            r#"{"field":{"field":[{"active":1},{"passive":1}]}}"#,
            "$.field.field[?(@.active)]",
            jp_v![&value;"$.['field'].['field'][0]",],
        );
    }

    #[test]
    fn index_index_test() {
        let value = json!("0-553-21311-3");
        test(
            template_json(),
            "$..book[2].isbn",
            jp_v![&value;"$.['store'].['book'][2].['isbn']",],
        );
    }

    #[test]
    fn index_unit_index_test() {
        let value = json!("0-553-21311-3");
        test(
            template_json(),
            "$..book[2,4].isbn",
            jp_v![&value;"$.['store'].['book'][2].['isbn']",],
        );
        let value1 = json!("0-395-19395-8");
        test(
            template_json(),
            "$..book[2,3].isbn",
            jp_v![&value;"$.['store'].['book'][2].['isbn']", &value1;"$.['store'].['book'][3].['isbn']",],
        );
    }

    #[test]
    fn index_unit_keys_test() {
        let js1 = json!("Moby Dick");
        let js2 = json!(8.99);
        let js3 = json!("The Lord of the Rings");
        let js4 = json!(22.99);
        test(
            template_json(),
            "$..book[2,3]['title','price']",
            jp_v![
                &js1;"$.['store'].['book'][2].['title']",
                &js2;"$.['store'].['book'][2].['price']",
                &js3;"$.['store'].['book'][3].['title']",
                &js4;"$.['store'].['book'][3].['price']",],
        );
    }

    #[test]
    fn index_slice_test() {
        let i0 = "$.['array'][0]";
        let i1 = "$.['array'][1]";
        let i2 = "$.['array'][2]";
        let i3 = "$.['array'][3]";
        let i4 = "$.['array'][4]";
        let i5 = "$.['array'][5]";
        let i6 = "$.['array'][6]";
        let i7 = "$.['array'][7]";
        let i8 = "$.['array'][8]";
        let i9 = "$.['array'][9]";

        let j0 = json!(0);
        let j1 = json!(1);
        let j2 = json!(2);
        let j3 = json!(3);
        let j4 = json!(4);
        let j5 = json!(5);
        let j6 = json!(6);
        let j7 = json!(7);
        let j8 = json!(8);
        let j9 = json!(9);
        test(
            template_json(),
            "$.array[:]",
            jp_v![
                &j0;&i0,
                &j1;&i1,
                &j2;&i2,
                &j3;&i3,
                &j4;&i4,
                &j5;&i5,
                &j6;&i6,
                &j7;&i7,
                &j8;&i8,
                &j9;&i9,],
        );
        test(template_json(), "$.array[1:4:2]", jp_v![&j1;&i1, &j3;&i3,]);
        test(
            template_json(),
            "$.array[::3]",
            jp_v![&j0;&i0, &j3;&i3, &j6;&i6, &j9;&i9,],
        );
        test(template_json(), "$.array[-1:]", jp_v![&j9;&i9,]);
        test(template_json(), "$.array[-2:-1]", jp_v![&j8;&i8,]);
    }

    #[test]
    fn index_filter_test() {
        let moby = json!("Moby Dick");
        let rings = json!("The Lord of the Rings");
        test(
            template_json(),
            "$..book[?(@.isbn)].title",
            jp_v![
                &moby;"$.['store'].['book'][2].['title']",
                &rings;"$.['store'].['book'][3].['title']",],
        );
        let sword = json!("Sword of Honour");
        test(
            template_json(),
            "$..book[?(@.price != 8.95)].title",
            jp_v![
                &sword;"$.['store'].['book'][1].['title']",
                &moby;"$.['store'].['book'][2].['title']",
                &rings;"$.['store'].['book'][3].['title']",],
        );
        let sayings = json!("Sayings of the Century");
        test(
            template_json(),
            "$..book[?(@.price == 8.95)].title",
            jp_v![&sayings;"$.['store'].['book'][0].['title']",],
        );
        let js895 = json!(8.95);
        test(
            template_json(),
            "$..book[?(@.author ~= '.*Rees')].price",
            jp_v![&js895;"$.['store'].['book'][0].['price']",],
        );
        let js12 = json!(12.99);
        let js899 = json!(8.99);
        let js2299 = json!(22.99);
        test(
            template_json(),
            "$..book[?(@.price >= 8.99)].price",
            jp_v![
                &js12;"$.['store'].['book'][1].['price']",
                &js899;"$.['store'].['book'][2].['price']",
                &js2299;"$.['store'].['book'][3].['price']",
            ],
        );
        test(
            template_json(),
            "$..book[?(@.price > 8.99)].price",
            jp_v![
                &js12;"$.['store'].['book'][1].['price']",
                &js2299;"$.['store'].['book'][3].['price']",],
        );
        test(
            template_json(),
            "$..book[?(@.price < 8.99)].price",
            jp_v![&js895;"$.['store'].['book'][0].['price']",],
        );
        test(
            template_json(),
            "$..book[?(@.price <= 8.99)].price",
            jp_v![
                &js895;"$.['store'].['book'][0].['price']",
                &js899;"$.['store'].['book'][2].['price']",
            ],
        );
        test(
            template_json(),
            "$..book[?(@.price <= $.expensive)].price",
            jp_v![
                &js895;"$.['store'].['book'][0].['price']",
                &js899;"$.['store'].['book'][2].['price']",
            ],
        );
        test(
            template_json(),
            "$..book[?(@.price >= $.expensive)].price",
            jp_v![
                &js12;"$.['store'].['book'][1].['price']",
                &js2299;"$.['store'].['book'][3].['price']",
            ],
        );
        test(
            template_json(),
            "$..book[?(@.title in ['Moby Dick','Shmoby Dick','Big Dick','Dicks'])].price",
            jp_v![&js899;"$.['store'].['book'][2].['price']",],
        );
        test(
            template_json(),
            "$..book[?(@.title nin ['Moby Dick','Shmoby Dick','Big Dick','Dicks'])].title",
            jp_v![
                &sayings;"$.['store'].['book'][0].['title']",
                &sword;"$.['store'].['book'][1].['title']",
                &rings;"$.['store'].['book'][3].['title']",],
        );
        test(
            template_json(),
            "$..book[?(@.author size 10)].title",
            jp_v![&sayings;"$.['store'].['book'][0].['title']",],
        );
        let filled_true = json!(1);
        test(
            template_json(),
            "$.orders[?(@.filled == true)].id",
            jp_v![&filled_true;"$.['orders'][0].['id']",],
        );
        let filled_null = json!(3);
        test(
            template_json(),
            "$.orders[?(@.filled == null)].id",
            jp_v![&filled_null;"$.['orders'][2].['id']",],
        );
    }

    #[test]
    fn index_filter_sets_test() {
        let j1 = json!(1);
        test(
            template_json(),
            "$.orders[?(@.ref subsetOf [1,2,3,4])].id",
            jp_v![&j1;"$.['orders'][0].['id']",],
        );
        let j2 = json!(2);
        test(
            template_json(),
            "$.orders[?(@.ref anyOf [1,4])].id",
            jp_v![&j1;"$.['orders'][0].['id']", &j2;"$.['orders'][1].['id']",],
        );
        let j3 = json!(3);
        test(
            template_json(),
            "$.orders[?(@.ref noneOf [3,6])].id",
            jp_v![&j3;"$.['orders'][2].['id']",],
        );
    }

    #[test]
    fn query_test() {
        let json: Box<Value> = serde_json::from_str(template_json()).expect("to get json");
        let v = json
            .path("$..book[?(@.author size 10)].title")
            .expect("the path is correct");
        assert_eq!(v, json!(["Sayings of the Century"]));

        let json: Value = serde_json::from_str(template_json()).expect("to get json");
        let path = &json
            .path("$..book[?(@.author size 10)].title")
            .expect("the path is correct");

        assert_eq!(path, &json!(["Sayings of the Century"]));
    }

    #[test]
    fn find_slice_test() {
        let json: Box<Value> = serde_json::from_str(template_json()).expect("to get json");
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$..book[?(@.author size 10)].title")
                .expect("the path is correct"),
        );
        let v = super::find_slice(&path, &json);
        let js = json!("Sayings of the Century");
        assert_eq!(v, jp_v![&js;"$.['store'].['book'][0].['title']",]);
    }

    #[test]
    fn find_in_array_test() {
        let json: Box<Value> = Box::new(json!([{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.[?(@.verb == 'TEST')]").expect("the path is correct"),
        );
        let v = super::find_slice(&path, &json);
        let js = json!({"verb":"TEST"});
        assert_eq!(v, jp_v![&js;"$[0]",]);
    }

    #[test]
    fn length_test() {
        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.[?(@.verb == 'TEST')].length()")
                .expect("the path is correct"),
        );
        let v = super::find(&path, &json);
        let js = json!([2]);
        assert_eq!(v, js);

        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPathInst> =
            Box::from(JsonPathInst::from_str("$.length()").expect("the path is correct"));
        assert_eq!(super::find(&path, &json), json!([3]));

        // length of search following the wildcard returns correct result
        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST","x":3}, {"verb": "RUN"}]));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.[?(@.verb == 'TEST')].[*].length()")
                .expect("the path is correct"),
        );
        assert_eq!(super::find(&path, &json), json!([3]));

        // length of object returns 0
        let json: Box<Value> = Box::new(json!({"verb": "TEST"}));
        let path: Box<JsonPathInst> =
            Box::from(JsonPathInst::from_str("$.length()").expect("the path is correct"));
        assert_eq!(super::find(&path, &json), Value::Null);

        // length of integer returns null
        let json: Box<Value> = Box::new(json!(1));
        let path: Box<JsonPathInst> =
            Box::from(JsonPathInst::from_str("$.length()").expect("the path is correct"));
        assert_eq!(super::find(&path, &json), Value::Null);

        // length of array returns correct result
        let json: Box<Value> = Box::new(json!([[1], [2], [3]]));
        let path: Box<JsonPathInst> =
            Box::from(JsonPathInst::from_str("$.length()").expect("the path is correct"));
        assert_eq!(super::find(&path, &json), json!([3]));

        // path does not exist returns length null
        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPathInst> =
            Box::from(JsonPathInst::from_str("$.not.exist.length()").expect("the path is correct"));
        assert_eq!(super::find(&path, &json), Value::Null);

        // seraching one value returns correct length
        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.[?(@.verb == 'RUN')].length()").expect("the path is correct"),
        );

        let v = super::find(&path, &json);
        let js = json!([1]);
        assert_eq!(v, js);

        // searching correct path following unexisting key returns length 0
        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.[?(@.verb == 'RUN')].key123.length()")
                .expect("the path is correct"),
        );

        let v = super::find(&path, &json);
        let js = json!(null);
        assert_eq!(v, js);

        // fetching first object returns length null
        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPathInst> =
            Box::from(JsonPathInst::from_str("$.[0].length()").expect("the path is correct"));

        let v = super::find(&path, &json);
        let js = Value::Null;
        assert_eq!(v, js);

        // length on fetching the index after search gives length of the object (array)
        let json: Box<Value> = Box::new(json!([{"prop": [["a", "b", "c"], "d"]}]));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.[?(@.prop)].prop.[0].length()").expect("the path is correct"),
        );

        let v = super::find(&path, &json);
        let js = json!([3]);
        assert_eq!(v, js);

        // length on fetching the index after search gives length of the object (string)
        let json: Box<Value> = Box::new(json!([{"prop": [["a", "b", "c"], "d"]}]));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.[?(@.prop)].prop.[1].length()").expect("the path is correct"),
        );

        let v = super::find(&path, &json);
        let js = Value::Null;
        assert_eq!(v, js);
    }

    #[test]
    fn no_value_index_from_not_arr_filter_test() {
        let json: Box<Value> = Box::new(json!({
            "field":"field",
        }));

        let path: Box<JsonPathInst> =
            Box::from(JsonPathInst::from_str("$.field[1]").expect("the path is correct"));
        let v = super::find_slice(&path, &json);
        assert_eq!(v, vec![NoValue]);

        let json: Box<Value> = Box::new(json!({
            "field":[0],
        }));

        let path: Box<JsonPathInst> =
            Box::from(JsonPathInst::from_str("$.field[1]").expect("the path is correct"));
        let v = super::find_slice(&path, &json);
        assert_eq!(v, vec![NoValue]);
    }

    #[test]
    fn no_value_filter_from_not_arr_filter_test() {
        let json: Box<Value> = Box::new(json!({
            "field":"field",
        }));

        let path: Box<JsonPathInst> =
            Box::from(JsonPathInst::from_str("$.field[?(@ == 0)]").expect("the path is correct"));
        let v = super::find_slice(&path, &json);
        assert_eq!(v, vec![NoValue]);
    }

    #[test]
    fn no_value_index_filter_test() {
        let json: Box<Value> = Box::new(json!({
            "field":[{"f":1},{"f":0}],
        }));

        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.field[?(@.f_ == 0)]").expect("the path is correct"),
        );
        let v = super::find_slice(&path, &json);
        assert_eq!(v, vec![NoValue]);
    }

    #[test]
    fn no_value_decent_test() {
        let json: Box<Value> = Box::new(json!({
            "field":[{"f":1},{"f":{"f_":1}}],
        }));

        let path: Box<JsonPathInst> =
            Box::from(JsonPathInst::from_str("$..f_").expect("the path is correct"));
        let v = super::find_slice(&path, &json);
        assert_eq!(
            v,
            vec![Slice(&json!(1), "$.['field'][1].['f'].['f_']".to_string())]
        );
    }

    #[test]
    fn no_value_chain_test() {
        let json: Box<Value> = Box::new(json!({
            "field":{"field":[1]},
        }));

        let path: Box<JsonPathInst> =
            Box::from(JsonPathInst::from_str("$.field_.field").expect("the path is correct"));
        let v = super::find_slice(&path, &json);
        assert_eq!(v, vec![NoValue]);

        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.field_.field[?(@ == 1)]").expect("the path is correct"),
        );
        let v = super::find_slice(&path, &json);
        assert_eq!(v, vec![NoValue]);
    }

    #[test]
    fn no_value_filter_test() {
        // searching unexisting value returns length 0
        let json: Box<Value> =
            Box::new(json!([{"verb": "TEST"},{"verb": "TEST"}, {"verb": "RUN"}]));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.[?(@.verb == \"RUN1\")]").expect("the path is correct"),
        );
        let v = super::find(&path, &json);
        let js = json!(null);
        assert_eq!(v, js);
    }

    #[test]
    fn no_value_len_test() {
        let json: Box<Value> = Box::new(json!({
            "field":{"field":1},
        }));

        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.field.field.length()").expect("the path is correct"),
        );
        let v = super::find_slice(&path, &json);
        assert_eq!(v, vec![NoValue]);

        let json: Box<Value> = Box::new(json!({
            "field":[{"a":1},{"a":1}],
        }));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.field[?(@.a == 0)].f.length()").expect("the path is correct"),
        );
        let v = super::find_slice(&path, &json);
        assert_eq!(v, vec![NoValue]);
    }

    #[test]
    fn no_clone_api_test() {
        fn test_coercion(value: &Value) -> Value {
            value.clone()
        }

        let json: Value = serde_json::from_str(template_json()).expect("to get json");
        let query = JsonPathInst::from_str("$..book[?(@.author size 10)].title")
            .expect("the path is correct");

        let results = query.find_slice(&json);
        let v = results.first().expect("to get value");

        // V can be implicitly converted to &Value
        test_coercion(v);

        // To explicitly convert to &Value, use deref()
        assert_eq!(v.deref(), &json!("Sayings of the Century"));
    }

    #[test]
    fn logical_exp_test() {
        let json: Box<Value> = Box::new(json!({"first":{"second":[{"active":1},{"passive":1}]}}));

        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.first[?(@.does_not_exist && @.does_not_exist >= 1.0)]")
                .expect("the path is correct"),
        );
        let v = super::find_slice(&path, &json);
        assert_eq!(v, vec![NoValue]);

        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.first[?(@.does_not_exist >= 1.0)]")
                .expect("the path is correct"),
        );
        let v = super::find_slice(&path, &json);
        assert_eq!(v, vec![NoValue]);
    }

    #[test]
    fn regex_filter_test() {
        let json: Box<Value> = Box::new(json!({
            "author":"abcd(Rees)",
        }));

        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.[?(@.author ~= '(?i)d\\(Rees\\)')]")
                .expect("the path is correct"),
        );
        assert_eq!(
            super::find_slice(&path, &json.clone()),
            vec![Slice(&json!({"author":"abcd(Rees)"}), "$".to_string())]
        );
    }

    #[test]
    fn logical_not_exp_test() {
        let json: Box<Value> = Box::new(json!({"first":{"second":{"active":1}}}));
        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.first[?(!@.does_not_exist >= 1.0)]")
                .expect("the path is correct"),
        );
        let v = super::find_slice(&path, &json);
        assert_eq!(
            v,
            vec![Slice(
                &json!({"second":{"active": 1}}),
                "$.['first']".to_string()
            )]
        );

        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.first[?(!(@.does_not_exist >= 1.0))]")
                .expect("the path is correct"),
        );
        let v = super::find_slice(&path, &json);
        assert_eq!(
            v,
            vec![Slice(
                &json!({"second":{"active": 1}}),
                "$.['first']".to_string()
            )]
        );

        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.first[?(!(@.second.active == 1) || @.second.active == 1)]")
                .expect("the path is correct"),
        );
        let v = super::find_slice(&path, &json);
        assert_eq!(
            v,
            vec![Slice(
                &json!({"second":{"active": 1}}),
                "$.['first']".to_string()
            )]
        );

        let path: Box<JsonPathInst> = Box::from(
            JsonPathInst::from_str("$.first[?(!@.second.active == 1 && !@.second.active == 1 || !@.second.active == 2)]")
                .expect("the path is correct"),
        );
        let v = super::find_slice(&path, &json);
        assert_eq!(
            v,
            vec![Slice(
                &json!({"second":{"active": 1}}),
                "$.['first']".to_string()
            )]
        );
    }

    // #[test]
    // fn no_value_len_field_test() {
    //     let json: Box<Value> =
    //         Box::new(json!([{"verb": "TEST","a":[1,2,3]},{"verb": "TEST","a":[1,2,3]},{"verb": "TEST"}, {"verb": "RUN"}]));
    //     let path: Box<JsonPathInst> = Box::from(
    //         JsonPathInst::from_str("$.[?(@.verb == 'TEST')].a.length()")
    //             .expect("the path is correct"),
    //     );
    //     let finder = JsonPathFinder::new(json, path);
    //
    //     let v = finder.find_slice();
    //     assert_eq!(v, vec![NewValue(json!(3))]);
    // }
}
