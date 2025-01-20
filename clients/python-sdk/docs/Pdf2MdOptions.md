# Pdf2MdOptions


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**split_headings** | **bool** | Split headings is an optional field which allows you to specify whether or not to split headings into separate chunks. Default is false. | [optional] 
**system_prompt** | **str** | Prompt to use for the gpt-4o model. Default is None. | [optional] 
**use_pdf2md_ocr** | **bool** | Parameter to use pdf2md_ocr. If true, the file will be converted to markdown using gpt-4o. Default is false. | 

## Example

```python
from trieve_py_client.models.pdf2_md_options import Pdf2MdOptions

# TODO update the JSON string below
json = "{}"
# create an instance of Pdf2MdOptions from a JSON string
pdf2_md_options_instance = Pdf2MdOptions.from_json(json)
# print the JSON string representation of the object
print(Pdf2MdOptions.to_json())

# convert the object into a dict
pdf2_md_options_dict = pdf2_md_options_instance.to_dict()
# create an instance of Pdf2MdOptions from a dict
pdf2_md_options_form_dict = pdf2_md_options.from_dict(pdf2_md_options_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


