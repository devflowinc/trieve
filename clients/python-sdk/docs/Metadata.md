# Metadata


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**article_section** | **str** |  | [optional] 
**article_tag** | **str** |  | [optional] 
**dc_date** | **str** |  | [optional] 
**dc_date_created** | **str** |  | [optional] 
**dc_description** | **str** |  | [optional] 
**dc_subject** | **str** |  | [optional] 
**dc_terms_audience** | **str** |  | [optional] 
**dc_terms_created** | **str** |  | [optional] 
**dc_terms_keywords** | **str** |  | [optional] 
**dc_terms_subject** | **str** |  | [optional] 
**dc_terms_type** | **str** |  | [optional] 
**dc_type** | **str** |  | [optional] 
**description** | **str** |  | [optional] 
**error** | **str** |  | [optional] 
**keywords** | **str** |  | [optional] 
**language** | **str** |  | [optional] 
**modified_time** | **str** |  | [optional] 
**og_audio** | **str** |  | [optional] 
**og_description** | **str** |  | [optional] 
**og_determiner** | **str** |  | [optional] 
**og_image** | **str** |  | [optional] 
**og_locale** | **str** |  | [optional] 
**og_locale_alternate** | **List[str]** |  | [optional] 
**og_site_name** | **str** |  | [optional] 
**og_title** | **str** |  | [optional] 
**og_url** | **str** |  | [optional] 
**og_video** | **str** |  | [optional] 
**published_time** | **str** |  | [optional] 
**robots** | **str** |  | [optional] 
**site_map** | [**Sitemap**](Sitemap.md) |  | [optional] 
**source_url** | **str** |  | [optional] 
**status_code** | **int** |  | [optional] 
**title** | **str** |  | [optional] 

## Example

```python
from trieve_py_client.models.metadata import Metadata

# TODO update the JSON string below
json = "{}"
# create an instance of Metadata from a JSON string
metadata_instance = Metadata.from_json(json)
# print the JSON string representation of the object
print(Metadata.to_json())

# convert the object into a dict
metadata_dict = metadata_instance.to_dict()
# create an instance of Metadata from a dict
metadata_form_dict = metadata.from_dict(metadata_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


