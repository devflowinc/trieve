# SplitHtmlResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunks** | [**List[ChunkedContent]**](ChunkedContent.md) |  | 

## Example

```python
from trieve_py_client.models.split_html_response import SplitHtmlResponse

# TODO update the JSON string below
json = "{}"
# create an instance of SplitHtmlResponse from a JSON string
split_html_response_instance = SplitHtmlResponse.from_json(json)
# print the JSON string representation of the object
print(SplitHtmlResponse.to_json())

# convert the object into a dict
split_html_response_dict = split_html_response_instance.to_dict()
# create an instance of SplitHtmlResponse from a dict
split_html_response_form_dict = split_html_response.from_dict(split_html_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


