# GetAllTagsReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**page** | **int** | Page number to return, 1-indexed. Default is 1. | [optional] 
**page_size** | **int** | Number of items to return per page. Default is 20. | [optional] 

## Example

```python
from trieve_py_client.models.get_all_tags_req_payload import GetAllTagsReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of GetAllTagsReqPayload from a JSON string
get_all_tags_req_payload_instance = GetAllTagsReqPayload.from_json(json)
# print the JSON string representation of the object
print(GetAllTagsReqPayload.to_json())

# convert the object into a dict
get_all_tags_req_payload_dict = get_all_tags_req_payload_instance.to_dict()
# create an instance of GetAllTagsReqPayload from a dict
get_all_tags_req_payload_form_dict = get_all_tags_req_payload.from_dict(get_all_tags_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


