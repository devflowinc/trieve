# CTRSearchQueryWithoutClicksResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**queries** | [**List[SearchQueriesWithoutClicksCTRResponse]**](SearchQueriesWithoutClicksCTRResponse.md) |  | 

## Example

```python
from trieve_py_client.models.ctr_search_query_without_clicks_response import CTRSearchQueryWithoutClicksResponse

# TODO update the JSON string below
json = "{}"
# create an instance of CTRSearchQueryWithoutClicksResponse from a JSON string
ctr_search_query_without_clicks_response_instance = CTRSearchQueryWithoutClicksResponse.from_json(json)
# print the JSON string representation of the object
print(CTRSearchQueryWithoutClicksResponse.to_json())

# convert the object into a dict
ctr_search_query_without_clicks_response_dict = ctr_search_query_without_clicks_response_instance.to_dict()
# create an instance of CTRSearchQueryWithoutClicksResponse from a dict
ctr_search_query_without_clicks_response_form_dict = ctr_search_query_without_clicks_response.from_dict(ctr_search_query_without_clicks_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


