# BulkDeleteChunkPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**ChunkFilter**](ChunkFilter.md) |  | 

## Example

```python
from trieve_py_client.models.bulk_delete_chunk_payload import BulkDeleteChunkPayload

# TODO update the JSON string below
json = "{}"
# create an instance of BulkDeleteChunkPayload from a JSON string
bulk_delete_chunk_payload_instance = BulkDeleteChunkPayload.from_json(json)
# print the JSON string representation of the object
print(BulkDeleteChunkPayload.to_json())

# convert the object into a dict
bulk_delete_chunk_payload_dict = bulk_delete_chunk_payload_instance.to_dict()
# create an instance of BulkDeleteChunkPayload from a dict
bulk_delete_chunk_payload_form_dict = bulk_delete_chunk_payload.from_dict(bulk_delete_chunk_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


