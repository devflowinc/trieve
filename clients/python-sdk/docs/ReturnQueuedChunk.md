# ReturnQueuedChunk


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_metadata** | [**List[ChunkMetadata]**](ChunkMetadata.md) |  | 
**pos_in_queue** | **int** | The current position the last access item is in the queue | 

## Example

```python
from trieve_py_client.models.return_queued_chunk import ReturnQueuedChunk

# TODO update the JSON string below
json = "{}"
# create an instance of ReturnQueuedChunk from a JSON string
return_queued_chunk_instance = ReturnQueuedChunk.from_json(json)
# print the JSON string representation of the object
print(ReturnQueuedChunk.to_json())

# convert the object into a dict
return_queued_chunk_dict = return_queued_chunk_instance.to_dict()
# create an instance of ReturnQueuedChunk from a dict
return_queued_chunk_form_dict = return_queued_chunk.from_dict(return_queued_chunk_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


