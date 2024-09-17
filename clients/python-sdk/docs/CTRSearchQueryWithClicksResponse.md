# CTRSearchQueryWithClicksResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**queries** | [**List[SearchQueriesWithClicksCTRResponse]**](SearchQueriesWithClicksCTRResponse.md) |  | 

## Example

```python
from trieve_py_client.models.ctr_search_query_with_clicks_response import CTRSearchQueryWithClicksResponse

# TODO update the JSON string below
json = "{}"
# create an instance of CTRSearchQueryWithClicksResponse from a JSON string
ctr_search_query_with_clicks_response_instance = CTRSearchQueryWithClicksResponse.from_json(json)
# print the JSON string representation of the object
print(CTRSearchQueryWithClicksResponse.to_json())

# convert the object into a dict
ctr_search_query_with_clicks_response_dict = ctr_search_query_with_clicks_response_instance.to_dict()
# create an instance of CTRSearchQueryWithClicksResponse from a dict
ctr_search_query_with_clicks_response_form_dict = ctr_search_query_with_clicks_response.from_dict(ctr_search_query_with_clicks_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


