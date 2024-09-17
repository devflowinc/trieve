# LocationPolygon


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**exterior** | [**List[GeoInfo]**](GeoInfo.md) |  | 
**interior** | **List[List[GeoInfo]]** |  | [optional] 

## Example

```python
from trieve_py_client.models.location_polygon import LocationPolygon

# TODO update the JSON string below
json = "{}"
# create an instance of LocationPolygon from a JSON string
location_polygon_instance = LocationPolygon.from_json(json)
# print the JSON string representation of the object
print(LocationPolygon.to_json())

# convert the object into a dict
location_polygon_dict = location_polygon_instance.to_dict()
# create an instance of LocationPolygon from a dict
location_polygon_form_dict = location_polygon.from_dict(location_polygon_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


