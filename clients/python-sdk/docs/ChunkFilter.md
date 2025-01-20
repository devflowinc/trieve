# ChunkFilter

ChunkFilter is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**must** | [**List[ConditionType]**](ConditionType.md) | All of these field conditions have to match for the chunk to be included in the result set. | [optional] 
**must_not** | [**List[ConditionType]**](ConditionType.md) | None of these field conditions can match for the chunk to be included in the result set. | [optional] 
**should** | [**List[ConditionType]**](ConditionType.md) | Only one of these field conditions has to match for the chunk to be included in the result set. | [optional] 

## Example

```python
from trieve_py_client.models.chunk_filter import ChunkFilter

# TODO update the JSON string below
json = "{}"
# create an instance of ChunkFilter from a JSON string
chunk_filter_instance = ChunkFilter.from_json(json)
# print the JSON string representation of the object
print(ChunkFilter.to_json())

# convert the object into a dict
chunk_filter_dict = chunk_filter_instance.to_dict()
# create an instance of ChunkFilter from a dict
chunk_filter_form_dict = chunk_filter.from_dict(chunk_filter_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


