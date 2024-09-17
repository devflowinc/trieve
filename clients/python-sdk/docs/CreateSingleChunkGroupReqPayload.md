# CreateSingleChunkGroupReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**description** | **str** | Description to assign to the chunk_group. Convenience field for you to avoid having to remember what the group is for. | [optional] 
**metadata** | **object** | Optional metadata to assign to the chunk_group. This is a JSON object that can store any additional information you want to associate with the chunks inside of the chunk_group. | [optional] 
**name** | **str** | Name to assign to the chunk_group. Does not need to be unique. | [optional] 
**tag_set** | **List[str]** | Optional tags to assign to the chunk_group. This is a list of strings that can be used to categorize the chunks inside the chunk_group. | [optional] 
**tracking_id** | **str** | Optional tracking id to assign to the chunk_group. This is a unique identifier for the chunk_group. | [optional] 
**upsert_by_tracking_id** | **bool** | Upsert when a chunk_group with the same tracking_id exists. By default this is false, and the request will fail if a chunk_group with the same tracking_id exists. If this is true, the chunk_group will be updated if a chunk_group with the same tracking_id exists. | [optional] 

## Example

```python
from trieve_py_client.models.create_single_chunk_group_req_payload import CreateSingleChunkGroupReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of CreateSingleChunkGroupReqPayload from a JSON string
create_single_chunk_group_req_payload_instance = CreateSingleChunkGroupReqPayload.from_json(json)
# print the JSON string representation of the object
print(CreateSingleChunkGroupReqPayload.to_json())

# convert the object into a dict
create_single_chunk_group_req_payload_dict = create_single_chunk_group_req_payload_instance.to_dict()
# create an instance of CreateSingleChunkGroupReqPayload from a dict
create_single_chunk_group_req_payload_form_dict = create_single_chunk_group_req_payload.from_dict(create_single_chunk_group_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


