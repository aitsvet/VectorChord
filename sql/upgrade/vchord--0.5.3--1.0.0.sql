CREATE FUNCTION vchordrq_list_centroids(regclass) RETURNS TABLE(level INT, id INT, centroid TEXT)
STRICT LANGUAGE c AS 'MODULE_PATHNAME', '_vchordrq_list_centroids_wrapper';
