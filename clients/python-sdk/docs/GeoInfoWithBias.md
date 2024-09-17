# GeoInfoWithBias

Location bias lets you rank your results by distance from a location. If not specified, this has no effect. Bias allows you to determine how much of an effect the location of chunks will have on the search results. If not specified, this defaults to 0.0. We recommend setting this to 1.0 for a gentle reranking of the results, >3.0 for a strong reranking of the results.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**bias** | **float** | Bias lets you specify how much of an effect the location of chunks will have on the search results. If not specified, this defaults to 0.0. We recommend setting this to 1.0 for a gentle reranking of the results, &gt;3.0 for a strong reranking of the results. | 
**location** | [**GeoInfo**](GeoInfo.md) |  | 

## Example

```python
from trieve_py_client.models.geo_info_with_bias import GeoInfoWithBias

# TODO update the JSON string below
json = "{}"
# create an instance of GeoInfoWithBias from a JSON string
geo_info_with_bias_instance = GeoInfoWithBias.from_json(json)
# print the JSON string representation of the object
print(GeoInfoWithBias.to_json())

# convert the object into a dict
geo_info_with_bias_dict = geo_info_with_bias_instance.to_dict()
# create an instance of GeoInfoWithBias from a dict
geo_info_with_bias_form_dict = geo_info_with_bias.from_dict(geo_info_with_bias_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


