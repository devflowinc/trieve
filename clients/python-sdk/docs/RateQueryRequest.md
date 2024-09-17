# RateQueryRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**note** | **str** |  | [optional] 
**query_id** | **str** |  | 
**rating** | **int** |  | 

## Example

```python
from trieve_py_client.models.rate_query_request import RateQueryRequest

# TODO update the JSON string below
json = "{}"
# create an instance of RateQueryRequest from a JSON string
rate_query_request_instance = RateQueryRequest.from_json(json)
# print the JSON string representation of the object
print(RateQueryRequest.to_json())

# convert the object into a dict
rate_query_request_dict = rate_query_request_instance.to_dict()
# create an instance of RateQueryRequest from a dict
rate_query_request_form_dict = rate_query_request.from_dict(rate_query_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


