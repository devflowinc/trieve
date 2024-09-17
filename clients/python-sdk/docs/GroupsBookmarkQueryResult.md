# GroupsBookmarkQueryResult


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunks** | [**List[ChunkMetadataStringTagSet]**](ChunkMetadataStringTagSet.md) |  | 
**group** | [**ChunkGroupAndFileId**](ChunkGroupAndFileId.md) |  | 
**total_pages** | **int** |  | 

## Example

```python
from trieve_py_client.models.groups_bookmark_query_result import GroupsBookmarkQueryResult

# TODO update the JSON string below
json = "{}"
# create an instance of GroupsBookmarkQueryResult from a JSON string
groups_bookmark_query_result_instance = GroupsBookmarkQueryResult.from_json(json)
# print the JSON string representation of the object
print(GroupsBookmarkQueryResult.to_json())

# convert the object into a dict
groups_bookmark_query_result_dict = groups_bookmark_query_result_instance.to_dict()
# create an instance of GroupsBookmarkQueryResult from a dict
groups_bookmark_query_result_form_dict = groups_bookmark_query_result.from_dict(groups_bookmark_query_result_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


