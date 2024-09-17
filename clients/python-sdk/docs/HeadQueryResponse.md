# HeadQueryResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**queries** | [**List[HeadQueries]**](HeadQueries.md) |  | 

## Example

```python
from trieve_py_client.models.head_query_response import HeadQueryResponse

# TODO update the JSON string below
json = "{}"
# create an instance of HeadQueryResponse from a JSON string
head_query_response_instance = HeadQueryResponse.from_json(json)
# print the JSON string representation of the object
print(HeadQueryResponse.to_json())

# convert the object into a dict
head_query_response_dict = head_query_response_instance.to_dict()
# create an instance of HeadQueryResponse from a dict
head_query_response_form_dict = head_query_response.from_dict(head_query_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


