# QueryCountResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**total_queries** | [**List[SearchTypeCount]**](SearchTypeCount.md) |  | 

## Example

```python
from trieve_py_client.models.query_count_response import QueryCountResponse

# TODO update the JSON string below
json = "{}"
# create an instance of QueryCountResponse from a JSON string
query_count_response_instance = QueryCountResponse.from_json(json)
# print the JSON string representation of the object
print(QueryCountResponse.to_json())

# convert the object into a dict
query_count_response_dict = query_count_response_instance.to_dict()
# create an instance of QueryCountResponse from a dict
query_count_response_form_dict = query_count_response.from_dict(query_count_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


