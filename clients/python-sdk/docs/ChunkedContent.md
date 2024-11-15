# ChunkedContent


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**body** | **str** | The body of the content | 
**headings** | **List[str]** | The headings of the content in order of when they appear | 

## Example

```python
from trieve_py_client.models.chunked_content import ChunkedContent

# TODO update the JSON string below
json = "{}"
# create an instance of ChunkedContent from a JSON string
chunked_content_instance = ChunkedContent.from_json(json)
# print the JSON string representation of the object
print(ChunkedContent.to_json())

# convert the object into a dict
chunked_content_dict = chunked_content_instance.to_dict()
# create an instance of ChunkedContent from a dict
chunked_content_form_dict = chunked_content.from_dict(chunked_content_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


