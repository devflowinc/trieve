# SingleProductOptions


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**group_tracking_id** | **str** |  | [optional] 
**product_description_html** | **str** |  | [optional] 
**product_name** | **str** |  | [optional] 
**product_primary_image_url** | **str** |  | [optional] 
**product_questions** | **List[str]** |  | [optional] 
**product_tracking_id** | **str** |  | [optional] 
**rec_search_query** | **str** |  | [optional] 

## Example

```python
from trieve_py_client.models.single_product_options import SingleProductOptions

# TODO update the JSON string below
json = "{}"
# create an instance of SingleProductOptions from a JSON string
single_product_options_instance = SingleProductOptions.from_json(json)
# print the JSON string representation of the object
print(SingleProductOptions.to_json())

# convert the object into a dict
single_product_options_dict = single_product_options_instance.to_dict()
# create an instance of SingleProductOptions from a dict
single_product_options_form_dict = single_product_options.from_dict(single_product_options_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


