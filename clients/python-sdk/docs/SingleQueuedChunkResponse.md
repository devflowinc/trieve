# SingleQueuedChunkResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_metadata** | [**ChunkMetadata**](ChunkMetadata.md) |  | 
**pos_in_queue** | **int** | The current position the last access item is in the queue | 

## Example

```python
from trieve_py_client.models.single_queued_chunk_response import SingleQueuedChunkResponse

# TODO update the JSON string below
json = "{}"
# create an instance of SingleQueuedChunkResponse from a JSON string
single_queued_chunk_response_instance = SingleQueuedChunkResponse.from_json(json)
# print the JSON string representation of the object
print(SingleQueuedChunkResponse.to_json())

# convert the object into a dict
single_queued_chunk_response_dict = single_queued_chunk_response_instance.to_dict()
# create an instance of SingleQueuedChunkResponse from a dict
single_queued_chunk_response_form_dict = single_queued_chunk_response.from_dict(single_queued_chunk_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


