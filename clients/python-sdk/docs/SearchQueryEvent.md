# SearchQueryEvent


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **str** |  | 
**dataset_id** | **str** |  | 
**id** | **str** |  | 
**latency** | **float** |  | 
**query** | **str** |  | 
**query_rating** | **str** |  | 
**request_params** | **object** |  | 
**results** | **List[object]** |  | 
**search_type** | **str** |  | 
**top_score** | **float** |  | 
**user_id** | **str** |  | 

## Example

```python
from trieve_py_client.models.search_query_event import SearchQueryEvent

# TODO update the JSON string below
json = "{}"
# create an instance of SearchQueryEvent from a JSON string
search_query_event_instance = SearchQueryEvent.from_json(json)
# print the JSON string representation of the object
print(SearchQueryEvent.to_json())

# convert the object into a dict
search_query_event_dict = search_query_event_instance.to_dict()
# create an instance of SearchQueryEvent from a dict
search_query_event_form_dict = search_query_event.from_dict(search_query_event_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


