# SearchResultType


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**highlights** | **List[str]** |  | [optional] 
**metadata** | [**List[ScoreChunkDTO]**](ScoreChunkDTO.md) |  | 
**score** | **float** |  | 
**file_id** | **str** |  | [optional] 
**group_created_at** | **datetime** |  | 
**group_dataset_id** | **str** |  | 
**group_description** | **str** |  | [optional] 
**group_id** | **str** |  | 
**group_metadata** | **object** |  | [optional] 
**group_name** | **str** |  | [optional] 
**group_tag_set** | **List[Optional[str]]** |  | [optional] 
**group_tracking_id** | **str** |  | [optional] 
**group_updated_at** | **datetime** |  | 

## Example

```python
from trieve_py_client.models.search_result_type import SearchResultType

# TODO update the JSON string below
json = "{}"
# create an instance of SearchResultType from a JSON string
search_result_type_instance = SearchResultType.from_json(json)
# print the JSON string representation of the object
print(SearchResultType.to_json())

# convert the object into a dict
search_result_type_dict = search_result_type_instance.to_dict()
# create an instance of SearchResultType from a dict
search_result_type_form_dict = search_result_type.from_dict(search_result_type_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


