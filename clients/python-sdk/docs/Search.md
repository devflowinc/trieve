# Search


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**event_type** | **str** |  | 
**latency** | **float** | Latency of the search | [optional] 
**query** | **str** | The search query | 
**query_rating** | [**SearchQueryRating**](SearchQueryRating.md) |  | [optional] 
**request_params** | **object** | The request params of the search | [optional] 
**results** | **List[object]** | The results of the search | [optional] 
**search_type** | [**ClickhouseSearchTypes**](ClickhouseSearchTypes.md) |  | [optional] 
**top_score** | **float** | The top score of the search | [optional] 
**user_id** | **str** | The user id of the user who made the search | [optional] 

## Example

```python
from trieve_py_client.models.search import Search

# TODO update the JSON string below
json = "{}"
# create an instance of Search from a JSON string
search_instance = Search.from_json(json)
# print the JSON string representation of the object
print(Search.to_json())

# convert the object into a dict
search_dict = search_instance.to_dict()
# create an instance of Search from a dict
search_form_dict = search.from_dict(search_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


