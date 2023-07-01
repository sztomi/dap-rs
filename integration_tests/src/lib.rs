#[cfg(test)]
mod integration_tests {
  use std::collections::HashSet;
  use serde_json::Value;
  use dap::prelude::*;
  use dap::responses::*;
  use jsonschema::JSONSchema;

  pub fn resp_to_value(resp: &Response) -> serde_json::Value {
    let msg = dap::base_message::BaseMessage {
      seq: 1,
      message: dap::base_message::Sendable::Response(resp.to_owned()),
    };
    serde_json::to_value(&msg).unwrap()
  }

  // This function reads a given definition from the schema and recursively replces all $ref values
  // with their expanded definitions. This is necessary because jsonschema doesn't seem to do this
  // automatically, and I can't find a way to make it do so.
  //
  // For example, consider resolving `Request`:
  // 
  //    "ProtocolMessage": {
  //    	"type": "object",
  //    	"title": "Base Protocol",
  //    	"description": "Base class of requests, responses, and events.",
  //    	"properties": {
  //    		"seq": {
  //    			"type": "integer",
  //    			"description": "Sequence number of the message (also known as message ID). The `seq` for the first message sent by a client or debug adapter is 1, and for each subsequent message is 1 greater than the previous message sent by that actor. `seq` can be used to order requests, responses, and events, and to associate requests with their corresponding responses. For protocol messages of type `request` the sequence number can be used to cancel the request."
  //    		},
  //    		"type": {
  //    			"type": "string",
  //    			"description": "Message type.",
  //    			"_enum": [ "request", "response", "event" ]
  //    		}
  //    	},
  //    	"required": [ "seq", "type" ]
  //    },
  //    "Request": {
  //    	"allOf": [ { "$ref": "#/definitions/ProtocolMessage" }, {
  //    		"type": "object",
  //    		"description": "A client or debug adapter initiated request.",
  //    		"properties": {
  //    			"type": {
  //    				"type": "string",
  //    				"enum": [ "request" ]
  //    			},
  //    			"command": {
  //    				"type": "string",
  //    				"description": "The command to execute."
  //    			},
  //    			"arguments": {
  //    				"type": [ "array", "boolean", "integer", "null", "number" , "object", "string" ],
  //    				"description": "Object containing arguments for the command."
  //    			}
  //    		},
  //    		"required": [ "type", "command" ]
  //    	}]
  //    },
  //
  // After resolving the $ref, the final definition of `Request` will be:
  //
  //    "Request":{
  //       "allOf":[
  //          {
  //             "type":"object",
  //             "title":"Base Protocol",
  //             "description":"Base class of requests, responses, and events.",
  //             "properties":{
  //                "seq":{
  //                   "type":"integer",
  //                   "description":"Sequence number of the message (also known as message ID). The `seq` for the first message sent by a client or debug adapter is 1, and for each subsequent message is 1 greater than the previous message sent by that actor. `seq` can be used to order requests, responses, and events, and to associate requests with their corresponding responses. For protocol messages of type `request` the sequence number can be used to cancel the request."
  //                },
  //                "type":{
  //                   "type":"string",
  //                   "description":"Message type.",
  //                   "_enum":[
  //                      "request",
  //                      "response",
  //                      "event"
  //                   ]
  //                }
  //             },
  //             "required":[
  //                "seq",
  //                "type"
  //             ]
  //          },
  //          {
  //             "type":"object",
  //             "description":"A client or debug adapter initiated request.",
  //             "properties":{
  //                "type":{
  //                   "type":"string",
  //                   "enum":[
  //                      "request"
  //                   ]
  //                },
  //                "command":{
  //                   "type":"string",
  //                   "description":"The command to execute."
  //                },
  //                "arguments":{
  //                   "type":[
  //                      "array",
  //                      "boolean",
  //                      "integer",
  //                      "null",
  //                      "number",
  //                      "object",
  //                      "string"
  //                   ],
  //                   "description":"Object containing arguments for the command."
  //                }
  //             },
  //             "required":[
  //                "type",
  //                "command"
  //             ]
  //          }
  //       ]
  //    }
  pub fn resolve_refs(schema_part: Value, full_schema: &Value, resolved_refs: &mut HashSet<String>) -> Value {
    match schema_part {
      Value::Object(map) => {
        let mut resolved = serde_json::Map::new();
        for (k, v) in map {
          if k == "$ref" {
            if let Value::String(ref_str) = v {
              if resolved_refs.contains(&ref_str) {
                continue;
              }
              resolved_refs.insert(ref_str.clone());

              let ref_parts = ref_str.split('/').skip(1); // skip initial #
              let mut ref_schema = full_schema;
              for part in ref_parts {
                ref_schema = ref_schema.get(part).unwrap();
              }
              if let Value::Object(ref_schema_map) = ref_schema {
                for (k, v) in ref_schema_map {
                  resolved.insert(k.clone(), resolve_refs(v.clone(), full_schema, resolved_refs));
                }
              }
            }
          } else {
            resolved.insert(k.clone(), resolve_refs(v, full_schema, resolved_refs));
          }
        }
        Value::Object(resolved)
      }
      Value::Array(vec) => Value::Array(
        vec
          .into_iter()
          .map(|v| resolve_refs(v, full_schema, resolved_refs))
          .collect(),
      ),
      _ => schema_part,
    }
  }

  pub fn get_schema(item: &str) -> Value {
    let schema = include_str!("../b01a8da52b83850c1a35e024bca09f7b285ac109_debugAdapterProtocol.json");
    let schema: Value = serde_json::from_str(schema).unwrap();
    let mut refs = HashSet::new();

    // Get the specific definition from the schema
    let schema_part = schema
      .get("definitions")
      .unwrap()
      .get(item)
      .unwrap()
      .clone();

    resolve_refs(schema_part, &schema, &mut refs)
  }

  include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
}
