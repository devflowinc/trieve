# CTRDataRequestBody


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**clicked_chunk_id** | **str** | The ID of chunk that was clicked | [optional] 
**clicked_chunk_tracking_id** | **str** | The tracking ID of the chunk that was clicked | [optional] 
**ctr_type** | [**CTRType**](CTRType.md) |  | 
**metadata** | **object** | Any metadata you want to include with the event i.e. action, user_id, etc. | [optional] 
**position** | **int** | The position of the clicked chunk | 
**request_id** | **str** | The request id for the CTR data | 

## Example

```python
from trieve_py_client.models.ctr_data_request_body import CTRDataRequestBody

# TODO update the JSON string below
json = "{}"
# create an instance of CTRDataRequestBody from a JSON string
ctr_data_request_body_instance = CTRDataRequestBody.from_json(json)
# print the JSON string representation of the object
print(CTRDataRequestBody.to_json())

# convert the object into a dict
ctr_data_request_body_dict = ctr_data_request_body_instance.to_dict()
# create an instance of CTRDataRequestBody from a dict
ctr_data_request_body_form_dict = ctr_data_request_body.from_dict(ctr_data_request_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


