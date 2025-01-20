# ChunkReqPayloadMapping

Express a mapping between a column or field in a CSV or JSONL field and a key in the ChunkReqPayload created for each row or object.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_req_payload_field** | [**ChunkReqPayloadFields**](ChunkReqPayloadFields.md) |  | 
**csv_jsonl_field** | **str** | The column or field in the CSV or JSONL file that you want to map to a key in the ChunkReqPayload | 

## Example

```python
from trieve_py_client.models.chunk_req_payload_mapping import ChunkReqPayloadMapping

# TODO update the JSON string below
json = "{}"
# create an instance of ChunkReqPayloadMapping from a JSON string
chunk_req_payload_mapping_instance = ChunkReqPayloadMapping.from_json(json)
# print the JSON string representation of the object
print(ChunkReqPayloadMapping.to_json())

# convert the object into a dict
chunk_req_payload_mapping_dict = chunk_req_payload_mapping_instance.to_dict()
# create an instance of ChunkReqPayloadMapping from a dict
chunk_req_payload_mapping_form_dict = chunk_req_payload_mapping.from_dict(chunk_req_payload_mapping_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


