# HasChunkIDCondition

HasChunkIDCondition is a JSON object which can be used to filter chunks by their ids or tracking ids. This is useful for when you want to filter chunks by their ids or tracking ids.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ids** | **List[str]** | Ids of the chunks to apply a match_any condition with. Only chunks with one of these ids will be returned. | [optional] 
**tracking_ids** | **List[str]** | Tracking ids of the chunks to apply a match_any condition with. Only chunks with one of these tracking ids will be returned. | [optional] 

## Example

```python
from trieve_py_client.models.has_chunk_id_condition import HasChunkIDCondition

# TODO update the JSON string below
json = "{}"
# create an instance of HasChunkIDCondition from a JSON string
has_chunk_id_condition_instance = HasChunkIDCondition.from_json(json)
# print the JSON string representation of the object
print(HasChunkIDCondition.to_json())

# convert the object into a dict
has_chunk_id_condition_dict = has_chunk_id_condition_instance.to_dict()
# create an instance of HasChunkIDCondition from a dict
has_chunk_id_condition_form_dict = has_chunk_id_condition.from_dict(has_chunk_id_condition_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


