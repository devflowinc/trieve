# CreateSchemaReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**include_images** | **bool** |  | [optional] 
**model** | **str** |  | [optional] 
**prompt** | **str** |  | 
**tag_enum** | **List[str]** |  | [optional] 

## Example

```python
from trieve_py_client.models.create_schema_req_payload import CreateSchemaReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of CreateSchemaReqPayload from a JSON string
create_schema_req_payload_instance = CreateSchemaReqPayload.from_json(json)
# print the JSON string representation of the object
print(CreateSchemaReqPayload.to_json())

# convert the object into a dict
create_schema_req_payload_dict = create_schema_req_payload_instance.to_dict()
# create an instance of CreateSchemaReqPayload from a dict
create_schema_req_payload_form_dict = create_schema_req_payload.from_dict(create_schema_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


