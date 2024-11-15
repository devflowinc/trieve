# RequestInfo


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**request_id** | **str** |  | 
**request_type** | [**CTRType**](CTRType.md) |  | 

## Example

```python
from trieve_py_client.models.request_info import RequestInfo

# TODO update the JSON string below
json = "{}"
# create an instance of RequestInfo from a JSON string
request_info_instance = RequestInfo.from_json(json)
# print the JSON string representation of the object
print(RequestInfo.to_json())

# convert the object into a dict
request_info_dict = request_info_instance.to_dict()
# create an instance of RequestInfo from a dict
request_info_form_dict = request_info.from_dict(request_info_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


