# BatchQueuedChunkResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_metadata** | [**List[ChunkMetadata]**](ChunkMetadata.md) |  | 
**pos_in_queue** | **int** | The current position the last access item is in the queue | 

## Example

```python
from trieve_py_client.models.batch_queued_chunk_response import BatchQueuedChunkResponse

# TODO update the JSON string below
json = "{}"
# create an instance of BatchQueuedChunkResponse from a JSON string
batch_queued_chunk_response_instance = BatchQueuedChunkResponse.from_json(json)
# print the JSON string representation of the object
print(BatchQueuedChunkResponse.to_json())

# convert the object into a dict
batch_queued_chunk_response_dict = batch_queued_chunk_response_instance.to_dict()
# create an instance of BatchQueuedChunkResponse from a dict
batch_queued_chunk_response_form_dict = batch_queued_chunk_response.from_dict(batch_queued_chunk_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


