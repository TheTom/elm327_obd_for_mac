# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
TX Wildlife POC - Complete Dockerized pipeline for trail camera analysis. Uses YOLOv8 detection + configurable classification (CLIP/ONNX) with custom training capabilities for Texas wildlife species identification.

## Development Commands

### **Quick Start**
```bash
make smoke          # Validate environment
make classify       # Run main pipeline
make clean          # Clear outputs
```

### **Classification Options**
```bash
make classify            # CLIP zero-shot (POC/baseline)
make onnx-classify      # ONNX wildlife models (production)
make wildlife-classify  # PyTorch-Wildlife (experimental)
```

### **Training Workflow**
```bash
make export-for-labeling # Export crops for manual labeling
make label-prep         # Organize labeled data
make build-split        # Create train/val split
make train-resnet       # Train custom classifier → ONNX
```

### **Development**
```bash
make build              # Standard build
make rebuild            # Force clean rebuild
make shell             # Access container shell
```

## Architecture Notes

### **Pipeline Flow**
1. **Detection**: YOLOv8n finds animals in images/videos
2. **Cropping**: Extract animal regions with padding
3. **Classification**: Species identification (CLIP/ONNX/Custom)
4. **Output**: CSV results + crops + visualizations

### **Input/Output**
- **Input**: `./data/samples/` - trail cam images (JPG/PNG) or videos (MP4/MOV)
- **Output**: 
  - `./data/out/results_*.csv` - classification results with confidence scores
  - `./data/out/crops/` - extracted animal crops
  - `./data/out/viz/` - detection visualizations with bounding boxes

### **Core Components**
- `src/detect_and_classify.py` - Main CLIP-based pipeline
- `src/detect_onnx_classify.py` - ONNX wildlife classifier pipeline  
- `src/export_for_labeling.py` - Export crops for manual labeling
- `src/labels_tx.py` - Texas wildlife species labels
- `src/tools/` - Training utilities (label_prep, build_split, train_resnet)

### **Configuration**
All settings tunable via `docker-compose.yml` environment variables:
- **Detection**: `DET_CONF`, `DET_IOU`, `MIN_CROP_PX`
- **Classification**: `MIN_SPECIES_P`, `MIN_MARGIN`, `MIN_CLIP_FLOOR`
- **Video**: `FRAME_STRIDE`, `MAX_FRAMES_PER_CLIP`

### **Key Features**
- **Environment-based tuning**: No code edits needed for threshold changes
- **Video optimization**: Frame stride + clip limits (90% speed improvement)
- **Quality filtering**: Skip tiny/poor crops
- **Model flexibility**: Easy swapping between CLIP/ONNX/custom models
- **Training pipeline**: Complete workflow from raw footage → trained classifier
- **Conservative uncertainty**: Proper handling of low-confidence predictions

## Training Data Organization

See `TRAINING.md` for complete workflow. Quick summary:

1. **Collect**: Drop trail cam footage in `data/samples/`
2. **Export**: `make export-for-labeling` extracts clean crops
3. **Label**: Organize crops by species or edit CSV
4. **Train**: `make train-resnet` creates custom ONNX classifier
5. **Deploy**: `make onnx-classify` uses your trained model

## Performance Notes

- **Docker**: CPU-only, good for POC/development
- **Native**: For production, run on Mac with MPS acceleration
- **CLIP**: ~30 seconds per crop (slow but works)
- **ONNX**: ~1 second per crop (fast with trained models)
- **Training**: ~10 minutes for 1000 samples on modern hardware