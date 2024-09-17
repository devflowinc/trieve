# SuggestedQueriesResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**queries** | **List[str]** |  | 

## Example

```python
from trieve_py_client.models.suggested_queries_response import SuggestedQueriesResponse

# TODO update the JSON string below
json = "{}"
# create an instance of SuggestedQueriesResponse from a JSON string
suggested_queries_response_instance = SuggestedQueriesResponse.from_json(json)
# print the JSON string representation of the object
print(SuggestedQueriesResponse.to_json())

# convert the object into a dict
suggested_queries_response_dict = suggested_queries_response_instance.to_dict()
# create an instance of SuggestedQueriesResponse from a dict
suggested_queries_response_form_dict = suggested_queries_response.from_dict(suggested_queries_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


