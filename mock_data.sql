
INSERT INTO checkup_projects (id, name, created_at, updated_at) VALUES ('P001', '血常规', '2024-02-13 10:00:00', '2024-02-13 10:00:00');
INSERT INTO checkup_projects (id, name, created_at, updated_at) VALUES ('P002', '肝功全套', '2024-02-13 10:00:00', '2024-02-13 10:00:00');

-- 20 Records
-- Record 1
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_001', '2024-02-13', 'ai_done', 'Simulated Note 1', '2024-02-13T10:00:00', '2024-02-13T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_001', 'rec_001', 'P001', 'img1.jpg', 'img1.jpg', '2024-02-13T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_001', 'file_001', 'rec_001', 'P001', '2024-02-13', '[{\
name\:\白细胞计数\,\value\:\12.5\,\unit\:\10^9/L\,\reference_range\:\3.5-9.5\,\is_abnormal\:true}]', 'success', '2024-02-13T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_001', 'rec_001', '### AI健康分析\n本次检查发现**白细胞计数偏高**，建议复查。', 'success', '2024-02-13T10:02:00');

-- Record 2
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_002', '2024-02-12', 'ai_done', 'Simulated Note 2', '2024-02-12T10:00:00', '2024-02-12T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_002', 'rec_002', 'P002', 'img2.jpg', 'img2.jpg', '2024-02-12T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_002', 'file_002', 'rec_002', 'P002', '2024-02-12', '[{\name\:\谷丙转氨酶\,\value\:\35\,\unit\:\U/L\,\reference_range\:\0-40\,\is_abnormal\:false}]', 'success', '2024-02-12T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_002', 'rec_002', '### AI健康分析\n各项指标正常，建议保持健康生活方式。', 'success', '2024-02-12T10:02:00');

-- Record 3
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_003', '2024-02-11', 'ai_done', 'Simulated Note 3', '2024-02-11T10:00:00', '2024-02-11T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_003', 'rec_003', 'P001', 'img3.jpg', 'img3.jpg', '2024-02-11T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_003', 'file_003', 'rec_003', 'P001', '2024-02-11', '[{\name\:\红细胞计数\,\value\:\3.0\,\unit\:\10^12/L\,\reference_range\:\3.5-5.5\,\is_abnormal\:true}]', 'success', '2024-02-11T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_003', 'rec_003', '### AI健康分析\n发现**红细胞计数偏低**，可能存在贫血风险，建议补充铁质。', 'success', '2024-02-11T10:02:00');

-- Record 4
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_004', '2024-02-10', 'ai_done', '', '2024-02-10T10:00:00', '2024-02-10T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_004', 'rec_004', 'P002', 'img4.jpg', 'img4.jpg', '2024-02-10T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_004', 'file_004', 'rec_004', 'P002', '2024-02-10', '[{\name\:\总胆红素\,\value\:\15\,\unit\:\umol/L\,\reference_range\:\3.4-17.1\,\is_abnormal\:false}]', 'success', '2024-02-10T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_004', 'rec_004', '指标正常。', 'success', '2024-02-10T10:02:00');

-- Record 5
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_005', '2024-02-09', 'ocr_done', '', '2024-02-09T10:00:00', '2024-02-09T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_005', 'rec_005', 'P001', 'img5.jpg', 'img5.jpg', '2024-02-09T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_005', 'file_005', 'rec_005', 'P001', '2024-02-09', '[{\name\:\血小板\,\value\:\200\,\unit\:\10^9/L\,\reference_range\:\100-300\,\is_abnormal\:false}]', 'success', '2024-02-09T10:01:00');
-- No AI analysis for this one

-- Record 6
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_006', '2024-02-08', 'ai_done', '', '2024-02-08T10:00:00', '2024-02-08T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_006', 'rec_006', 'P001', 'img6.jpg', 'img6.jpg', '2024-02-08T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_006', 'file_006', 'rec_006', 'P001', '2024-02-08', '[{\name\:\中性粒细胞\,\value\:\80\,\unit\:\%\,\reference_range\:\50-70\,\is_abnormal\:true}]', 'success', '2024-02-08T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_006', 'rec_006', '发现轻微细菌感染迹象。', 'success', '2024-02-08T10:02:00');

-- Record 7
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_007', '2024-02-07', 'ai_done', '', '2024-02-07T10:00:00', '2024-02-07T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_007', 'rec_007', 'P002', 'img7.jpg', 'img7.jpg', '2024-02-07T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_007', 'file_007', 'rec_007', 'P002', '2024-02-07', '[{\name\:\白蛋白\,\value\:\45\,\unit\:\g/L\,\reference_range\:\40-55\,\is_abnormal\:false}]', 'success', '2024-02-07T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_007', 'rec_007', '肝功能正常。', 'success', '2024-02-07T10:02:00');

-- Record 8
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_008', '2024-02-06', 'ai_done', '', '2024-02-06T10:00:00', '2024-02-06T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_008', 'rec_008', 'P001', 'img8.jpg', 'img8.jpg', '2024-02-06T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_008', 'file_008', 'rec_008', 'P001', '2024-02-06', '[]', 'success', '2024-02-06T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_008', 'rec_008', '未检测到明显指标。', 'success', '2024-02-06T10:02:00');

-- Record 9
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_009', '2024-02-05', 'ai_done', '', '2024-02-05T10:00:00', '2024-02-05T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_009', 'rec_009', 'P002', 'img9.jpg', 'img9.jpg', '2024-02-05T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_009', 'file_009', 'rec_009', 'P002', '2024-02-05', '[{\name\:\谷草转氨酶\,\value\:\50\,\unit\:\U/L\,\reference_range\:\0-40\,\is_abnormal\:true}]', 'success', '2024-02-05T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_009', 'rec_009', '谷草转氨酶略高，注意休息。', 'success', '2024-02-05T10:02:00');

-- Record 10
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_010', '2024-02-04', 'ai_done', '', '2024-02-04T10:00:00', '2024-02-04T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_010', 'rec_010', 'P001', 'img10.jpg', 'img10.jpg', '2024-02-04T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_010', 'file_010', 'rec_010', 'P001', '2024-02-04', '[{\name\:\淋巴细胞\,\value\:\40\,\unit\:\%\,\reference_range\:\20-40\,\is_abnormal\:false}]', 'success', '2024-02-04T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_010', 'rec_010', '一切正常。', 'success', '2024-02-04T10:02:00');

-- Record 11
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_011', '2024-02-03', 'ai_done', '', '2024-02-03T10:00:00', '2024-02-03T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_011', 'rec_011', 'P002', 'img11.jpg', 'img11.jpg', '2024-02-03T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_011', 'file_011', 'rec_011', 'P002', '2024-02-03', '[]', 'success', '2024-02-03T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_011', 'rec_011', '无异常。', 'success', '2024-02-03T10:02:00');

-- Record 12
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_012', '2024-02-02', 'ai_done', '', '2024-02-02T10:00:00', '2024-02-02T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_012', 'rec_012', 'P001', 'img12.jpg', 'img12.jpg', '2024-02-02T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_012', 'file_012', 'rec_012', 'P001', '2024-02-02', '[]', 'success', '2024-02-02T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_012', 'rec_012', '无异常。', 'success', '2024-02-02T10:02:00');

-- Record 13
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_013', '2024-02-01', 'ai_done', '', '2024-02-01T10:00:00', '2024-02-01T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_013', 'rec_013', 'P002', 'img13.jpg', 'img13.jpg', '2024-02-01T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_013', 'file_013', 'rec_013', 'P002', '2024-02-01', '[{\name\:\碱性磷酸酶\,\value\:\150\,\unit\:\U/L\,\reference_range\:\45-125\,\is_abnormal\:true}]', 'success', '2024-02-01T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_013', 'rec_013', '碱性磷酸酶偏高。', 'success', '2024-02-01T10:02:00');

-- Record 14
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_014', '2024-01-31', 'ai_done', '', '2024-01-31T10:00:00', '2024-01-31T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_014', 'rec_014', 'P001', 'img14.jpg', 'img14.jpg', '2024-01-31T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_014', 'file_014', 'rec_014', 'P001', '2024-01-31', '[]', 'success', '2024-01-31T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_014', 'rec_014', '正常。', 'success', '2024-01-31T10:02:00');

-- Record 15
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_015', '2024-01-30', 'ai_done', '', '2024-01-30T10:00:00', '2024-01-30T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_015', 'rec_015', 'P002', 'img15.jpg', 'img15.jpg', '2024-01-30T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_015', 'file_015', 'rec_015', 'P002', '2024-01-30', '[]', 'success', '2024-01-30T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_015', 'rec_015', '正常。', 'success', '2024-01-30T10:02:00');

-- Record 16
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_016', '2024-01-29', 'ai_done', '', '2024-01-29T10:00:00', '2024-01-29T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_016', 'rec_016', 'P002', 'img16.jpg', 'img16.jpg', '2024-01-29T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_016', 'file_016', 'rec_016', 'P002', '2024-01-29', '[]', 'success', '2024-01-29T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_016', 'rec_016', '正常。', 'success', '2024-01-29T10:02:00');

-- Record 17
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_017', '2024-01-28', 'ai_done', '', '2024-01-28T10:00:00', '2024-01-28T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_017', 'rec_017', 'P001', 'img17.jpg', 'img17.jpg', '2024-01-28T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_017', 'file_017', 'rec_017', 'P001', '2024-01-28', '[]', 'success', '2024-01-28T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_017', 'rec_017', '正常。', 'success', '2024-01-28T10:02:00');

-- Record 18
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_018', '2024-01-27', 'ai_done', '', '2024-01-27T10:00:00', '2024-01-27T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_018', 'rec_018', 'P002', 'img18.jpg', 'img18.jpg', '2024-01-27T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_018', 'file_018', 'rec_018', 'P002', '2024-01-27', '[]', 'success', '2024-01-27T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_018', 'rec_018', '正常。', 'success', '2024-01-27T10:02:00');

-- Record 19
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_019', '2024-01-26', 'ai_done', '', '2024-01-26T10:00:00', '2024-01-26T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_019', 'rec_019', 'P002', 'img19.jpg', 'img19.jpg', '2024-01-26T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_019', 'file_019', 'rec_019', 'P002', '2024-01-26', '[]', 'success', '2024-01-26T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_019', 'rec_019', '正常。', 'success', '2024-01-26T10:02:00');

-- Record 20
INSERT INTO checkup_records (id, checkup_date, status, notes, created_at, updated_at) VALUES ('rec_020', '2024-01-25', 'ai_done', '', '2024-01-25T10:00:00', '2024-01-25T10:00:00');
INSERT INTO checkup_files (id, record_id, project_id, original_filename, stored_path, uploaded_at) VALUES ('file_020', 'rec_020', 'P001', 'img20.jpg', 'img20.jpg', '2024-01-25T10:00:00');
INSERT INTO ocr_results (id, file_id, record_id, project_id, checkup_date, parsed_items, status, created_at) VALUES ('ocr_020', 'file_020', 'rec_020', 'P001', '2024-01-25', '[]', 'success', '2024-01-25T10:01:00');
INSERT INTO ai_analyses (id, record_id, response_content, status, created_at) VALUES ('ai_020', 'rec_020', '正常。', 'success', '2024-01-25T10:02:00');

