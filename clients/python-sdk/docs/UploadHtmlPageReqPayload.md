# UploadHtmlPageReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**data** | [**Document**](Document.md) |  | 
**metadata** | **object** |  | 
**scrape_id** | **str** |  | 

## Example

```python
from trieve_py_client.models.upload_html_page_req_payload import UploadHtmlPageReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of UploadHtmlPageReqPayload from a JSON string
upload_html_page_req_payload_instance = UploadHtmlPageReqPayload.from_json(json)
# print the JSON string representation of the object
print(UploadHtmlPageReqPayload.to_json())

# convert the object into a dict
upload_html_page_req_payload_dict = upload_html_page_req_payload_instance.to_dict()
# create an instance of UploadHtmlPageReqPayload from a dict
upload_html_page_req_payload_form_dict = upload_html_page_req_payload.from_dict(upload_html_page_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


