# SearchQueryResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**queries** | [**List[SearchQueryEvent]**](SearchQueryEvent.md) |  | 

## Example

```python
from trieve_py_client.models.search_query_response import SearchQueryResponse

# TODO update the JSON string below
json = "{}"
# create an instance of SearchQueryResponse from a JSON string
search_query_response_instance = SearchQueryResponse.from_json(json)
# print the JSON string representation of the object
print(SearchQueryResponse.to_json())

# convert the object into a dict
search_query_response_dict = search_query_response_instance.to_dict()
# create an instance of SearchQueryResponse from a dict
search_query_response_form_dict = search_query_response.from_dict(search_query_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


